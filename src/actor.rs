/// place-holder trait for an actor, this might change at some point
pub trait Actor {
    fn init() -> Self
    where
        Self: Sized;
}

/// thing that couples the actor and the mailbox together
pub struct ActorCell {
    actor: Box<dyn Actor>,
    mailbox: Vec<Message>,
}

pub type Message = String;

pub struct ActorAddress;
