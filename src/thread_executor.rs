use crossbeam_channel::{bounded, Receiver, Sender};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

use crate::actor::{Actor, ActorAddress};
use crate::executor::{ExecutionContext, ExecutorCommands, ExecutorFactory};

const COMMAND_BUFFER_SIZE: usize = 32;

pub struct ThreadExecutorFactory {}
impl ExecutorFactory for ThreadExecutorFactory {
    fn spawn_executor(&self, name: String) -> Sender<ExecutorCommands> {
        let (sender, receiver) = bounded::<ExecutorCommands>(COMMAND_BUFFER_SIZE);
        thread::spawn(move || {
            ThreadExecutor {
                name,
                actors: HashMap::new(),
            }
            .run(receiver)
        });
        sender
    }
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
    fn run(&mut self, receiver: Receiver<ExecutorCommands>) {
        loop {
            if !receiver.is_empty() {
                match receiver.recv().unwrap() {
                    ExecutorCommands::SpawnActor(actor, name) => {
                        // let actor = type_id
                        //     .downcast_ref::<dyn Actor>()
                        //     .unwrap()
                        //     .init();
                        // self.actors.insert(name, actor);
                        self.actors.insert(name, actor);
                        println!("received spawn actor command");
                    }
                    ExecutorCommands::Shutdown => {
                        println!("received shutdown command");
                        break;
                    }
                }
                // TODO: process messages
            }
            println!("nothing to do, sleeping...");
            thread::sleep(Duration::from_millis(100));
        }
    }

    fn spawn_actor<A: Actor + 'static>(&mut self, name: String) -> ActorAddress {
        self.actors.insert(name.clone(), Box::new(A::init()));
        self.get_address(&name)
    }
}
