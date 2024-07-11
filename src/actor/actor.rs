use crate::actor::{ActorAddress, Envelope, SenderType};
use crate::error::BusanError;
use crate::executor::ExecutorCommands;
use crate::message::{Message, ToMessage};
use crate::system::RuntimeManagerRef;
use crate::util::lib_macros::channel_send;
use crate::util::CommandChannel;
use crossbeam_channel::Receiver;
use log::{trace, warn};
use std::rc::Weak;

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

    /// Hook called after shutdown has been initiated, but before the actor has been removed
    /// from the executor. This can be used to send any final messages before the actor is
    /// stopped.
    ///
    /// Cleanup may also happen in this method, but may also happen in the [`Actor::after_stop`]
    /// method.
    ///
    /// __Note:__ When an actor is stopped, new messages may not be received. So while
    /// messages can be sent, no reply will be able to be processed.
    fn before_stop(&mut self, _ctx: Context) {}

    /// Hook called after the actor has been shutdown and removed from the executor. This is
    /// the final chance to cleanup/close any resources before the actor is dropped.
    ///
    /// __Note:__ Messages can no longer be sent from this method. See [`Actor::before_stop`]
    /// for the last chance to send messages.
    fn after_stop(&mut self) {}
}

type ReceiverFunc = fn(&mut dyn Actor, Context, Box<dyn Message>);

/// ActorInit defines a method of construction for an actor that takes an initialization
/// message. This provides type-safe initialization of an actor while keeping construction
/// and internal state within the actor system.
pub trait ActorInit {
    type Init: Message;

    fn init(init_msg: Self::Init) -> Self
    where
        Self: Sized + Actor;
}

/// Simple type-alias for a bitmask representing the state of an [`ActorCell`].
pub type CellState = u8;

pub mod cell_state {
    use super::CellState;

    const SHUTDOWN: CellState = 0b0000_0001;

    /// Check if the cell is in a shutdown state
    pub fn is_shutdown(state: CellState) -> bool {
        state & SHUTDOWN == SHUTDOWN
    }

    /// Set the shell into a shutdown state. __Note__ that this is a one-way action. A cell
    /// in a shutdown state cannot be removed from the shutdown state.
    pub fn set_shutdown(state: &mut CellState) {
        *state |= SHUTDOWN;
    }
}

/// [`ActorCell`] is the wrapper to the user-defined actor, wrapping the mailbox parent references,
/// and other actor-related information that is useful internally. This is primarily an internal
/// interface, but is exposed for user-provided executors or extensions.
/// <!-- TODO: Is this actually useful for extension or does it need to be opened up more? -->
pub struct ActorCell {
    pub(crate) actor: Box<dyn Actor>,
    pub(crate) mailbox: Receiver<Envelope>,
    pub(crate) address: ActorAddress,
    pub(crate) children: Vec<ActorAddress>,
    pub(crate) parent: Option<ActorAddress>,
    pub(crate) state: CellState,
    pub(crate) ack_nonce: u32,

    pub(crate) receiver: &ReceiverFunc,
}

impl ActorCell {
    pub(crate) fn new(
        actor: Box<dyn Actor>,
        mailbox: Receiver<Envelope>,
        address: ActorAddress,
        parent: Option<ActorAddress>,
    ) -> Self {
        let receiver: ReceiverFunc = actor.as_ref().receive;
        Self {
            actor,
            mailbox,
            address,
            children: Vec::new(),
            parent,
            state: 0,
            ack_nonce: 0,
            receiver,
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
    pub(crate) executor_command_channel: &'a CommandChannel<ExecutorCommands>,
    pub(crate) parent: &'a Option<ActorAddress>,
    pub(crate) children: &'a mut Vec<ActorAddress>,
    pub(crate) sender: &'a SenderType,
    pub(crate) cell_state: &'a mut CellState,
    pub(crate) ack_nonce: &'a mut u32,
}

impl Context<'_> {
    /// Create a new (child) actor.
    ///
    /// This method will create a new actor instance and coordinate with the runtime manager to
    /// schedule the actor (on an executor). Whether local or remote, this is __not__ an immediate
    /// action as an available executor must be found or network communication with another instance
    /// must occur.
    ///
    /// A spawn handle is returned that can be used to poll that status or block on the actor being
    /// scheduled and ready to receive messages. The result of the `await_*` functions is a fully
    /// resolved `ActorAddress`
    pub fn spawn_child<A: ActorInit<Init = M> + Actor + 'static, T: ToMessage<M>, M: Message>(
        &mut self,
        name: &str,
        init_msg: T,
    ) -> ActorSpawnHandle {
        let address = ActorAddress::new_child(self.address, name, self.children.len());
        self.children.push(address.clone());
        let ready_channel = self.runtime_manager.assign_actor(
            Box::new(A::init(init_msg.to_message())),
            address,
            Some(self.address.clone()),
        );

        ActorSpawnHandle { ready_channel }
    }

    // TODO: Document
    // TODO: Coordinate documentation with `send` method
    pub fn send_message(
        &self,
        addr: &ActorAddress,
        mut message: Box<dyn Message>,
        ack_nonce: Option<u32>,
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

        let message = debug_serialize_msg!(message);

        // Send the message to the resolved address
        addr.send(Some(self.address.clone()), message, ack_nonce);
    }

    // TODO: Document
    // TODO: Talk about the debug_serialize_msg! in the docs
    pub fn send<M: Message + 'static, T: ToMessage<M>>(&self, addr: &ActorAddress, message: T) {
        let message = message.to_message();
        self.send_message(addr, Box::new(message), None);
    }

    // TODO: Document
    pub fn send_with_ack<M: Message + 'static, T: ToMessage<M>>(
        &mut self,
        addr: &ActorAddress,
        message: T,
    ) -> u32 {
        let nonce = *self.ack_nonce;
        *self.ack_nonce += 1;
        let message = message.to_message();
        self.send_message(addr, Box::new(message), Some(nonce));
        nonce
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

    pub fn address(&self) -> &ActorAddress {
        self.address
    }

    /// Perform immediate shutdown for the current actor.
    pub fn shutdown(&mut self) {
        cell_state::set_shutdown(self.cell_state);
        channel_send!(
            self.executor_command_channel,
            ExecutorCommands::ShutdownActor(self.address.clone())
        );
    }
}

pub struct ActorSpawnHandle {
    ready_channel: Receiver<Result<ActorAddress, BusanError>>,
}

/// Handle used to await actor assignment upon initial creation (spawn).
impl ActorSpawnHandle {
    /// Polls to see if the actor has been scheduled. This does not block. Once
    /// this returns `true`, an [`ActorAddress`] can be resolved with
    /// (`await_ready`)[`Self::await_ready`] without blocking.
    ///
    /// Note: Calling this function after the spawn handle has resolved to an
    /// [`ActorAddress`] (by calling (`await_ready`)[`Self::await_ready`] or
    /// (`await_unwrap`)[`Self::await_unwrap`]) has undefined behavior.
    pub fn ready(&self) -> bool {
        !self.ready_channel.is_empty()
    }

    /// Waits for the actor to be scheduled and returns an [`ActorAddress`] wrapped
    /// in a `Result`.
    ///
    /// This returns an error when the actor fails to be scheduled.
    ///
    /// Note: This function blocks for an indeterminate amount of time while awaiting
    /// actor assignment.
    pub fn await_ready(&self) -> Result<ActorAddress, BusanError> {
        // TODO: A reasonable wait timeout here would be good, probably something
        // that could be defined is a global/system-level config.
        match self.ready_channel.recv() {
            Ok(res) => res,
            Err(e) => Err(BusanError::UnassignableActor(format!(
                "Internal Error: {}",
                e
            ))),
        }
    }

    /// Blocks on actor assignment and discards the error. Equivalent to
    /// `spawn_handle.await_ready().unwrap()`
    pub fn await_unwrap(&self) -> ActorAddress {
        let res = self.await_ready();
        // If there is an error, panic with a descriptive message of what went wrong
        if let Err(e) = res {
            panic!("Failed while waiting for actor spawn: {}", e);
        }
        self.await_ready().unwrap()
    }
}
