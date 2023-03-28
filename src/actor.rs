use crate::message::Message;
use crate::system::RuntimeManagerRef;
use crossbeam_channel::{Receiver, Sender};
use log::warn;
use std::cell::RefCell;
use std::fmt::{Display, Formatter};

pub trait Actor: Send {
    fn before_start(&mut self, _ctx: Context) {}

    /// A receive for unhandled messages. Since message sending is untyped on the sender side,
    /// the receiver is typed, __and__ the receiver may have dynamic behaviours, is it possible
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
            msg.encoded_len()
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
    pub(crate) mailbox: Receiver<Box<dyn Message>>,
    pub(crate) address: ActorAddress,

    // Count of children that the actor has spawned. This is used to ensure that the actor names
    // are unique, with the value being appended to each child name and incremented on each use.
    pub(crate) child_count: usize,
}

impl ActorCell {
    pub fn new(
        actor: Box<dyn Actor>,
        mailbox: Receiver<Box<dyn Message>>,
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
        addr.send(message);
    }
}

#[derive(Debug)]
pub struct ActorAddress {
    pub(crate) uri: String,

    /// mailbox is a RefCell containing an optional sender. ActorAddresses may be created from
    /// just a path, but once a message is sent that path will need to resolve to a mailbox. Once
    /// the mailbox is resolved, it can be stored here for future use.
    pub(crate) mailbox: RefCell<Option<Sender<Box<dyn Message>>>>,
}

impl Display for ActorAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.uri)
    }
}

impl Clone for ActorAddress {
    fn clone(&self) -> Self {
        Self {
            uri: self.uri.clone(),
            mailbox: RefCell::new(self.mailbox.borrow().clone()),
        }
    }
}

impl ActorAddress {
    pub(crate) fn new_child(parent: &ActorAddress, name: &String, id: usize) -> Self {
        let uri = format!("{}/{}-{}", parent.uri, name, id);
        Self {
            uri,
            mailbox: RefCell::new(None),
        }
    }

    pub(crate) fn new_root(name: &String) -> Self {
        let uri = format!("local:/{}", name);
        Self {
            uri,
            mailbox: RefCell::new(None),
        }
    }

    pub(crate) fn set_mailbox(&self, mailbox: Sender<Box<dyn Message>>) {
        *self.mailbox.borrow_mut() = Some(mailbox);
    }

    pub(crate) fn is_resolved(&self) -> bool {
        self.mailbox.borrow().is_some()
    }

    pub(crate) fn send(&self, message: Box<dyn Message>) {
        let result = (&self.mailbox.borrow().as_ref().unwrap()).send(message);
        // TODO: Handle a non-OK error (once actor shutdown is implemented) On error, should
        //       redirect to the dead letter queue. This function may simply return an error
        //       so that the caller can do the redirection.
        debug_assert!(result.is_ok(), "Error sending to actor address {}", self);
    }
}
