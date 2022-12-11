use crossbeam_channel::{bounded, Receiver, Sender};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

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
    // Spawn an executor with a given name. Tha name will be used by the
    // executor for routing messages to the correct actor.
    fn spawn_executor(&self, name: &str) -> Sender<ExecutorCommands>;
}

pub struct ThreadExecutorFactory {}
impl ExecutorFactory for ThreadExecutorFactory {
    fn spawn_executor(&self, name: &str) -> Sender<ExecutorCommands> {
        let (sender, receiver) = bounded::<ExecutorCommands>(COMMAND_BUFFER_SIZE);
        thread::spawn(move || {
            ThreadExecutor {
                name: name.to_string(),
                actors: HashMap::new(),
            }.run(receiver)
        });
        sender
    }
}

pub trait ExecutionContext {
    fn run(&self, receiver: Receiver<ExecutorCommands>);
}

/// thread-local executor responsible for processing messages on actors
pub struct ThreadExecutor {
    // Name of the executor, which is part of the address to actors and used
    // for routing decisions
    name: String,

    // map of actor names to actors
    // the "name" is part of the address
    actors: HashMap<String, Box<dyn Actor>>,
}
impl ExecutionContext for ThreadExecutor {
    fn run(&self, receiver: Receiver<ExecutorCommands>) {
        loop {
            if !receiver.is_empty() {
                // TODO: process messages
            }
            thread::sleep(Duration::from_millis(100));
        }
    }
}
