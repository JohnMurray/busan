use crossbeam_channel::{bounded, Receiver};
use log::trace;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

use crate::actor::{Actor, ActorAddress};
use crate::executor::{Executor, ExecutorCommands, ExecutorFactory, ExecutorHandle};

const COMMAND_BUFFER_SIZE: usize = 32;

pub struct ThreadExecutorFactory {}
impl ExecutorFactory for ThreadExecutorFactory {
    fn spawn_executor(&self, name: String) -> ExecutorHandle {
        let (sender, receiver) = bounded::<ExecutorCommands>(COMMAND_BUFFER_SIZE);
        let t = thread::spawn(move || {
            ThreadExecutor {
                name,
                actors: HashMap::new(),
            }
            .run(receiver)
        });
        ExecutorHandle::new(sender, move || t.join().unwrap())
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
impl Executor for ThreadExecutor {
    fn run(&mut self, receiver: Receiver<ExecutorCommands>) {
        const SLEEP_DURATION_MS: u64 = 10;

        loop {
            if !receiver.is_empty() {
                match receiver.recv().unwrap() {
                    ExecutorCommands::SpawnActor(actor, name) => {
                        self.actors.insert(name, actor);
                        trace!("received spawn actor command");
                    }
                    ExecutorCommands::Shutdown => {
                        trace!("received shutdown command");
                        break;
                    }
                }
                // TODO: process messages
            }
            trace!("nothing to do, sleeping...");
            thread::sleep(Duration::from_millis(SLEEP_DURATION_MS));
        }
    }

    // fn spawn_actor<A: Actor + 'static>(&mut self, name: String) -> ActorAddress {
    //     self.actors.insert(name.clone(), Box::new(A::init()));
    //     self.get_address(&name)
    // }
}
