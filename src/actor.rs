use crate::executor::Executor;

/// place-holder trait for an actor, this might change at some point
pub trait Actor: Send {
    // fn init(init_msg: &dyn prost::Message) -> Self
    // where
    //     Self: Sized;

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
pub(crate) struct ActorCell {
    parent: Option<ActorAddress>,
    actor: Box<dyn Actor>,
    mailbox: Vec<Box<dyn prost::Message>>,
}

impl ActorCell {}

/// Actor context object used for performing actions that interact with the running
/// actor-system, such as spawning new actors.
pub struct Context<'a> {
    executor: &'a mut dyn Executor,
}

impl Context<'_> {
    pub fn new(executor: &mut dyn Executor) -> Context {
        Context { executor }
    }

    /// Create a new (child) actor. Note that this may be a delayed action and the actor
    /// may not be created immediately.
    pub fn spawn_child<B, A: ActorInit<Init = B> + Actor + 'static>(
        &self,
        name: String,
        init_msg: &B,
    ) -> ActorAddress {
        self.executor
            .assign_actor(Box::new(A::init(init_msg)), name.clone());
        self.executor.get_address(&name)
    }
}

#[derive(Clone)]
pub struct ActorAddress {
    pub name: String,
    pub executor_name: String,
}
