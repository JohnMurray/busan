use crossbeam_channel::{Receiver, Sender};

use crate::actor::{Actor, ActorAddress};

pub enum ExecutorCommands {
    SpawnActor(Box<dyn Actor>, String),
    Shutdown,
}

/// responsible for creating an executor
pub trait ExecutorFactory {
    // Spawn an executor with a given name. Tha name will be used by the
    // executor for routing messages to the correct actor.
    fn spawn_executor(&self, name: String) -> Sender<ExecutorCommands>;
}

pub trait Executor {
    fn run(&mut self, receiver: Receiver<ExecutorCommands>);
    // fn spawn_actor<A: Actor + 'static>(&mut self, name: String) -> ActorAddress;
}
