use crate::actor::{ActorAddress, Letter, SenderType};
use crate::executor::ExecutorCommands;
use crate::message::{Message, ToMessage};
use crate::system::RuntimeManagerRef;
use crate::util::CommandChannel;
use crossbeam_channel::Receiver;
use log::{trace, warn};

/// Trait that defines the behavior of an actor. This is the primary interface that must be
/// implemented when defining an actor.
pub trait Actor: Send {
    /// Hook called before any messages are received and after the actor has been initialized
    /// and assigned to an executor. This is useful for performing any initialization that
    /// requires the actor to be running.
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

    /// Receive a message. This is the primary method for handling messages and is called
    /// for every message received by the actor.
    fn receive(&mut self, ctx: Context, msg: Box<dyn Message>);

    /// Hook called after all messages have been received and processed and before the actor is
    /// removed fom the executor. This is useful for performing any cleanup that requires the
    /// actor to be running.
    ///
    /// Any messages sent to this actor after this method is called will be sent to the dead-letter
    /// queue.
    fn before_shutdown(&mut self, _ctx: Context) {}
}

/// ActorInit defines a method of construction for an actor that takes an initialization
/// message. This provides type-safe initialization of an actor while keeping construction
/// and internal state within the actor system.
pub trait ActorInit {
    type Init: Message;

    fn init(init_msg: Self::Init) -> Self
    where
        Self: Sized + Actor;
}

/// [`ActorCell`] is the wrapper to the user-defined actor, wrapping the mailbox parent references,
/// and other actor-related information that is useful internally. This is primarily an internal
/// interface, but is exposed for user-provided executors or extensions.
/// <!-- TODO: Is this actually useful for extension or does it need to be opened up more? -->
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

/// Debug macro for serializing and deserializing a message. The goal is to reduce
/// (or protect against) accidental state sharing on sent messages. Since the goal
/// is to support location transparency, it's beneficial to ensure state sharing
/// does not become part of the relied-upon behavior of an actor.
#[doc(hidden)]
macro_rules! debug_serialize_msg {
    ($msg:expr) => {
        if cfg!(debug_assertions) {
            let bytes = $msg.encode_to_vec2();
            $msg.merge2(bytes.as_slice()).unwrap();
            $msg
        } else {
            $msg
        }
    };
}

/// Actor context object used for performing actions that interact with the running
/// actor-system, such as spawning new actors and sending messages.
pub struct Context<'a> {
    pub(crate) address: &'a ActorAddress,
    pub(crate) runtime_manager: &'a RuntimeManagerRef,
    pub(crate) executor_channel: &'a CommandChannel<ExecutorCommands>,
    pub(crate) parent: &'a Option<ActorAddress>,
    pub(crate) children: &'a mut Vec<ActorAddress>,
    pub(crate) sender: &'a SenderType,
}

impl Context<'_> {
    /// Create a new (child) actor.
    ///
    /// This method will create a new actor instance and coordinate with the runtime manager to
    /// schedule the actor (on an executor). The address returned can be used to immediately send
    /// messages.
    ///
    /// __Important Note__ \
    /// Sending a message to an actor requires "resolving" the address. This is a blocking call
    /// to the runtime manager. Immediately creating and then sending a message to a child actor
    /// may result in a small amount of waiting. This should be avoided if possible.
    pub fn spawn_child<A: ActorInit<Init = M> + Actor + 'static, T: ToMessage<M>, M: Message>(
        &mut self,
        name: &str,
        init_msg: T,
    ) -> ActorAddress {
        let address = ActorAddress::new_child(self.address, name, self.children.len());
        self.children.push(address.clone());
        self.runtime_manager.assign_actor(
            Box::new(A::init(init_msg.to_message())),
            address.clone(),
            Some(self.address.clone()),
        );
        address
    }

    // TODO: Document
    // TODO: Coordinate documentation with `send` method
    pub fn send_message(&self, addr: &ActorAddress, mut message: Box<dyn Message>) {
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

        let message = debug_serialize_msg!(message);

        // Send the message to the resolved address
        addr.send(Some(self.address.clone()), message);
    }

    // TODO: Document
    // TODO: Talk about the debug_serialize_msg! in the docs
    pub fn send<M: Message + 'static, T: ToMessage<M>>(&self, addr: &ActorAddress, message: T) {
        let message = message.to_message();
        self.send_message(addr, Box::new(message));
    }

    /// Get the sender of the current message.
    ///
    /// __Note:__ If the message is a system message, there is no defined sender and this method
    /// will panic.
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

    /// Return the addresses for all children of the current actor
    pub fn children(&self) -> &[ActorAddress] {
        self.children
    }

    /// Return the address for the parent of the current actor. This will only return `None`
    /// for the root actor.
    pub fn parent(&self) -> Option<&ActorAddress> {
        self.parent.as_ref()
    }

    pub fn shutdown(&self) {
        self.executor_channel
            .send(ExecutorCommands::ShutdownActor(self.address.clone()))
            .unwrap();
    }
}
