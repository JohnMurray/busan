use std::any::{TypeId, Any};
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
            system.executors.insert(format!("executor-{}", i),system.executor_factory.spawn_executor(format!("executor-{}", i)));
        }

        system
    }

    pub fn spawn_actor<A: Actor + 'static>(&self, name: String) {
        assert!(self.executors.len() > 0, "No executors available to spawn actor");
        let type_id = TypeId::of::<A>();
        self.executors.get("executor-0").unwrap().send(ExecutorCommands::SpawnActor(type_id, name)).unwrap();
    }

    /// Send shutdown message to all executors and wait for them to finish
    pub fn wait_shutdown(&self) {
        for (_, executor) in self.executors.iter() {
            executor.send(ExecutorCommands::Shutdown).unwrap();
        }

        thread::sleep(Duration::from_millis(10_000));
    }

}

pub enum ExecutorCommands {
    SpawnActor(TypeId, String),
    Shutdown,
}

/// responsible for creating an executor
pub trait ExecutorFactory {
    // Spawn an executor with a given name. Tha name will be used by the
    // executor for routing messages to the correct actor.
    fn spawn_executor(&self, name: String) -> Sender<ExecutorCommands>;
}

pub struct ThreadExecutorFactory {}
impl ExecutorFactory for ThreadExecutorFactory {
    fn spawn_executor(&self, name: String) -> Sender<ExecutorCommands> {
        let (sender, receiver) = bounded::<ExecutorCommands>(COMMAND_BUFFER_SIZE);
        thread::spawn(move || {
            ThreadExecutor {
                name,
                actors: HashMap::new(),
            }.run(receiver)
        });
        sender
    }
}

pub trait ExecutionContext {
    fn run(&self, receiver: Receiver<ExecutorCommands>);
    fn spawn_actor<A: Actor + 'static
    >(&mut self, name: String) -> ActorAddress;
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
impl ThreadExecutor {
    fn get_address(&self, actor_name: &str) -> ActorAddress {
        ActorAddress {
            name: actor_name.to_string(),
            executor_name: self.name.clone(),
        }
    }
}
impl ExecutionContext for ThreadExecutor {
    fn run(&self, receiver: Receiver<ExecutorCommands>) {
        loop {
            if !receiver.is_empty() {
                match receiver.recv().unwrap() {
                    ExecutorCommands::SpawnActor(type_id, name) => {
                        // let actor = type_id
                        //     .downcast_ref::<dyn Actor>()
                        //     .unwrap()
                        //     .init();
                        // self.actors.insert(name, actor);
                        println!("received spawn actor command");
                    }
                    ExecutorCommands::Shutdown => {
                        println!("received shutdown command");
                        break;
                    }
                }
                // TODO: process messages
            }
            thread::sleep(Duration::from_millis(100));
        }
    }

    fn spawn_actor<A: Actor + 'static>(&mut self, name: String) -> ActorAddress {
        self.actors.insert(name.clone(), Box::new(A::init()));
        self.get_address(&name)
    }
}
