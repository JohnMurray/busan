use crate::system::RuntimeManagerRef;
use crossbeam_channel::{Receiver, Sender};
use std::cell::RefCell;

/// place-holder trait for an actor, this might change at some point
pub trait Actor: Send {
    fn before_start(&mut self, _ctx: Context) {}
}

/// ActorInit defines a method of construction for an actor that takes an initialization
/// message. This provides type-safe initialization of an actor while keeping construction
/// and internal state within the actor system.
pub trait ActorInit {
    type Init: prost::Message;

    fn init(init_msg: &Self::Init) -> Self
    where
        Self: Sized + Actor;
}

// NOTE:
//   - Sending messages should always be a prost::Message
//   - Receiving messages could be an Any type which _should_ allow for better pattern matching
//     against expected types. See:
//     https://stackoverflow.com/questions/26126683/how-to-match-trait-implementors

/// ActorCell is the wrapper to the user-defined actor, wrapping the mailbox parent references,
/// and other actor-related information that is useful internally.
pub struct ActorCell {
    pub(crate) actor: Box<dyn Actor>,
    pub(crate) mailbox: Receiver<Box<dyn prost::Message>>,
    pub(crate) address: ActorAddress,
}

impl ActorCell {
    pub fn new(
        actor: Box<dyn Actor>,
        mailbox: Receiver<Box<dyn prost::Message>>,
        address: ActorAddress,
    ) -> Self {
        Self {
            actor,
            mailbox,
            address,
        }
    }
}

/// Actor context object used for performing actions that interact with the running
/// actor-system, such as spawning new actors.
pub struct Context<'a, 'b> {
    pub(crate) address: &'a ActorAddress,
    pub(crate) runtime_manager: &'b RuntimeManagerRef,
}

impl Context<'_, '_> {
    /// Create a new (child) actor. Note that this may be a delayed action and the actor
    /// may not be created immediately.
    /// TODO: Ensure that actor names are unique
    pub fn spawn_child<B, A: ActorInit<Init = B> + Actor + 'static>(
        &self,
        name: String,
        init_msg: &B,
    ) -> ActorAddress {
        let address = ActorAddress::new_child(self.address, &name);
        self.runtime_manager
            .assign_actor(Box::new(A::init(init_msg)), address.clone());
        address
    }
}

#[derive(Debug)]
pub struct ActorAddress {
    pub(crate) uri: String,

    /// mailbox is a RefCell containing an optional sender. ActorAddresses may be created from
    /// just a path, but once a message is sent that path will need to resolve to a mailbox. Once
    /// the mailbox is resolved, it can be stored here for future use.
    pub(crate) mailbox: RefCell<Option<Sender<Box<dyn prost::Message>>>>,
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
    pub(crate) fn new_child(parent: &ActorAddress, name: &String) -> Self {
        let uri = format!("{}/{}", parent.uri, name);
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

    pub(crate) fn set_mailbox(&self, mailbox: Sender<Box<dyn prost::Message>>) {
        *self.mailbox.borrow_mut() = Some(mailbox);
    }
}
