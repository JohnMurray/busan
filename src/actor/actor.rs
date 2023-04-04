use crate::actor::{ActorAddress, Letter, SenderType};
use crate::message::Message;
use crate::system::RuntimeManagerRef;
use crossbeam_channel::Receiver;
use log::warn;

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
        // TODO: Log the sender of the message once the sender is available via the context
        warn!(
            "{}: unhandled message: ({} bytes)",
            ctx.address.uri,
            Message::encoded_len(msg.as_ref()),
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

// NOTE:
//   - Sending messages should always be a Message
//   - Receiving messages could be an Any type which _should_ allow for better pattern matching
//     against expected types. See:
//     https://stackoverflow.com/questions/26126683/how-to-match-trait-implementors

/// ActorCell is the wrapper to the user-defined actor, wrapping the mailbox parent references,
/// and other actor-related information that is useful internally.
pub struct ActorCell {
    pub(crate) actor: Box<dyn Actor>,
    pub(crate) mailbox: Receiver<Letter>,
    pub(crate) address: ActorAddress,

    // Count of children that the actor has spawned. This is used to ensure that the actor names
    // are unique, with the value being appended to each child name and incremented on each use.
    pub(crate) child_count: usize,
}

impl ActorCell {
    pub(crate) fn new(
        actor: Box<dyn Actor>,
        mailbox: Receiver<Letter>,
        address: ActorAddress,
    ) -> Self {
        Self {
            actor,
            mailbox,
            address,
            child_count: 0,
        }
    }
}

/// Actor context object used for performing actions that interact with the running
/// actor-system, such as spawning new actors.
pub struct Context<'a> {
    pub(crate) address: &'a ActorAddress,
    pub(crate) runtime_manager: &'a RuntimeManagerRef,
    pub(crate) child_count: &'a mut usize,
    pub(crate) sender: &'a SenderType,
}

impl Context<'_> {
    /// Create a new (child) actor. Note that this may be a delayed action and the actor
    /// may not be created immediately.
    /// TODO: Ensure that actor names are unique
    pub fn spawn_child<B, A: ActorInit<Init = B> + Actor + 'static>(
        &mut self,
        name: String,
        init_msg: &B,
    ) -> ActorAddress {
        let address = ActorAddress::new_child(self.address, &name, *self.child_count);
        *(self.child_count) += 1;
        self.runtime_manager
            .assign_actor(Box::new(A::init(init_msg)), address.clone());
        address
    }

    pub fn send_message(&self, addr: &ActorAddress, message: Box<dyn Message>) {
        // Validate that the address is resolved (this is a blocking call to the runtime
        // manager if unresolved).
        if !addr.is_resolved() {
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

        // Send the message to the resolved address
        addr.send(Some(self.address.clone()), message);
    }

    pub fn sender(&self) -> ActorAddress {
        match self.sender {
            SenderType::Actor(sender_address) => sender_address.clone(),
            _ => {
                todo!("Cannot currently get address from non-actor sender");
            }
        }
    }
}
