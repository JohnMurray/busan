use crate::actor::{ActorAddress, Letter, SenderType};
use crate::message::{Message, ToMessage};
use crate::system::RuntimeManagerRef;
use crossbeam_channel::Receiver;
use log::{trace, warn};

pub trait Actor: Send {
    fn before_start(&mut self, _ctx: Context) {}

    /// A receive for unhandled messages. Since message sending is untyped on the sender side,
    /// the receiver is typed, __and__ the receiver may have dynamic behaviors, is it possible
    /// that the actor is not capable of understanding or processing the given message. When this
    /// is the case, the message should be handed off to this method.
    ///
    /// Currently this method is roughly a no-op and simply emits a warning message. In the future,
    /// this will likely be used to forward the message to a dead letter queue.
    fn unhandled(&mut self, ctx: Context, msg: Box<dyn Message>) {
        warn!(
            "{}: unhandled message ({} bytes) sent from {}",
            ctx.address.uri,
            Message::encoded_len(msg.as_ref()),
            ctx.sender,
        );
    }

    fn receive(&mut self, ctx: Context, msg: Box<dyn Message>);
}

/// ActorInit defines a method of construction for an actor that takes an initialization
/// message. This provides type-safe initialization of an actor while keeping construction
/// and internal state within the actor system.
pub trait ActorInit {
    type Init: Message;

    fn init(init_msg: &Self::Init) -> Self
    where
        Self: Sized + Actor;
}

/// ActorCell is the wrapper to the user-defined actor, wrapping the mailbox parent references,
/// and other actor-related information that is useful internally.
pub struct ActorCell {
    pub(crate) actor: Box<dyn Actor>,
    pub(crate) mailbox: Receiver<Letter>,
    pub(crate) address: ActorAddress,
    pub(crate) children: Vec<ActorAddress>,
    pub(crate) parent: Option<ActorAddress>,
}

impl ActorCell {
    pub(crate) fn new(
        actor: Box<dyn Actor>,
        mailbox: Receiver<Letter>,
        address: ActorAddress,
        parent: Option<ActorAddress>,
    ) -> Self {
        Self {
            actor,
            mailbox,
            address,
            children: Vec::new(),
            parent,
        }
    }
}

macro_rules! debug_serialize_msg {
    ($msg:expr, $T:tt) => {
        if cfg!(debug_assertions) {
            let bytes = $msg.encode_to_vec2();
            $T::decode(bytes.as_slice()).unwrap()
        } else {
            $msg
        }
    };
}

/// Actor context object used for performing actions that interact with the running
/// actor-system, such as spawning new actors.
pub struct Context<'a> {
    pub(crate) address: &'a ActorAddress,
    pub(crate) runtime_manager: &'a RuntimeManagerRef,
    pub(crate) parent: &'a Option<ActorAddress>,
    pub(crate) children: &'a mut Vec<ActorAddress>,
    pub(crate) sender: &'a SenderType,
}

impl Context<'_> {
    /// Create a new (child) actor. Note that this may be a delayed action and the actor
    /// may not be created immediately.
    pub fn spawn_child<B, A: ActorInit<Init = B> + Actor + 'static>(
        &mut self,
        name: &str,
        init_msg: &B,
    ) -> ActorAddress {
        let address = ActorAddress::new_child(self.address, name, self.children.len());
        self.children.push(address.clone());
        self.runtime_manager.assign_actor(
            Box::new(A::init(init_msg)),
            address.clone(),
            Some(self.address.clone()),
        );
        address
    }

    pub fn send_message<M: Message + Default + 'static, T: ToMessage<M>>(
        &self,
        addr: &ActorAddress,
        message: T,
    ) {
        // Validate that the address is resolved (this is a blocking call to the runtime
        // manager if unresolved).
        if !addr.is_resolved() {
            trace!("Resolving address: {}", addr);
            match self.runtime_manager.resolve_address(addr) {
                Some(resolved) => {
                    addr.set_mailbox(resolved);
                }
                _ => {
                    todo!("Send message to dead letter queue");
                }
            }
        }

        // We should _either_ have a resolved address _OR_ the message should have been
        // forwarded to the dead letter queue.
        debug_assert!(addr.is_resolved(), "Address {} is not resolved", addr);

        let message = debug_serialize_msg!(message.to_message(), M);

        // Send the message to the resolved address
        addr.send(Some(self.address.clone()), Box::new(message));
    }

    pub fn sender(&self) -> &'_ ActorAddress {
        match self.sender {
            SenderType::Actor(sender_address) => sender_address,
            SenderType::Parent => {
                if let Some(parent) = self.parent {
                    return parent;
                }
                panic!("Message sent from parent, but no parent sender found")
            }
            SenderType::System => {
                panic!("Cannot retrieve a sender for system messages");
            }
            SenderType::SentToSelf => self.address,
        }
    }

    pub fn children(&self) -> &[ActorAddress] {
        self.children
    }
}
