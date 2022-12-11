use crossbeam_channel::{bounded, Receiver, Sender};
use std::collections::HashMap;

use crate::actor::{Actor, ActorAddress, Message};

const NUM_EXECUTORS: usize = 4;
const COMMAND_BUFFER_SIZE: usize = 32;

/// Global value for a collection of actors that may communicate with local-only addresses.
pub struct ActorSystem {
    executor_factory: Box<dyn ExecutorFactory>,
    executors: HashMap<String, Sender<ExecutorCommands>>,
}

impl ActorSystem {
    pub fn init() -> ActorSystem {
        // create a mutable actor system with a thread executor factory
        let mut system = ActorSystem {
            executor_factory: Box::new(ThreadExecutorFactory{}),
            executors: HashMap::new(),
        };

        // create a pre-configured number of executors
        for i in 0..NUM_EXECUTORS {
            system.executors.insert(format!("executor-{}", i),system.executor_factory.spawn_executor());
        }

        system
    }
}

pub enum ExecutorCommands {}

/// responsible for creating an executor
pub trait ExecutorFactory {
    // fn spawn_executors(): HashMap<int, >
    fn spawn_executor(&self) -> Sender<ExecutorCommands>;
}

pub struct ThreadExecutorFactory {}
impl ExecutorFactory for ThreadExecutorFactory {
    fn spawn_executor(&self) -> Sender<ExecutorCommands> {
        let (sender, _) = bounded::<ExecutorCommands>(COMMAND_BUFFER_SIZE);
        // TODO: spawn thread with the receiver
        sender
    }
}

/// thread-local executor responsible for processing messages on actors
pub trait ExecutionContext {
    // handler for dispatching/routing messages
    // TODO: Should this be a channel? So the executor can loop over it's global mailbox
    //       and route messages to sub-mailboxes?
    //
    // TODO: If we have a global channel, then why not just have each mailbox contain a
    //       channel that can be routed _directly_ to? Does that break any sort of encapsulation
    //       of the actor/actor-cell?
    fn route_msg(self: &Self, destination: ActorAddress, message: Message);
}

pub struct ThreadExecutor {
    // map of actor names to actors
    // the "name" is part of the address
    actors: HashMap<String, Box<dyn Actor>>,
}
impl ExecutionContext for ThreadExecutor {
    fn route_msg(self: &Self, destination: ActorAddress, message: Message) {
        todo!();
    }
}
