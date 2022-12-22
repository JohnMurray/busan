/// place-holder trait for an actor, this might change at some point
pub trait Actor: Send {
    // fn init(init_msg: &dyn prost::Message) -> Self
    // where
    //     Self: Sized;
}

// TODO: this name sucks
pub trait ActorInit {
    type Init: prost::Message;

    fn init(init_msg: &Self::Init) -> Self
    where
        Self: Sized + Actor;
}

// TODO:
//   1. Make an actor-creator/actor-initializer/actor-factory trait that contains the type
//      that is initializes the actor and holds the init method that is called wit the
//      correct type.
// NOTE:
//   - Sending messages should always be a prost::Message
//   - Receiving messages could be an Any type which _should_ allow for better pattern matching
//     against expected types. See:
//     https://stackoverflow.com/questions/26126683/how-to-match-trait-implementors

/// thing that couples the actor and the mailbox together
pub struct ActorCell {
    actor: Box<dyn Actor>,
    mailbox: Vec<Box<dyn prost::Message>>,
}

pub struct ActorAddress {
    pub name: String,
    pub executor_name: String,
}
