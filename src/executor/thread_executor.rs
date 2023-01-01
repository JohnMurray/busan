use crossbeam_channel::{bounded, Receiver, Sender};
use log::{debug, trace};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

use crate::actor::{Actor, ActorAddress, Context};
use crate::executor::{Executor, ExecutorCommands, ExecutorFactory, ExecutorHandle};

const COMMAND_BUFFER_SIZE: usize = 32;

pub struct ThreadExecutorFactory {}
impl ExecutorFactory for ThreadExecutorFactory {
    fn spawn_executor(&self, name: String) -> ExecutorHandle {
        let (sender, receiver) = bounded::<ExecutorCommands>(COMMAND_BUFFER_SIZE);
        let sender_copy = sender.clone();
        let t = thread::spawn(move || {
            ThreadExecutor {
                name,
                actors: HashMap::new(),
                sender: sender_copy,
            }
            .run(receiver)
        });
        ExecutorHandle::new(sender, move || t.join().unwrap())
    }
}

/// thread-local executor responsible for processing messages on actors
struct ThreadExecutor {
    // Name of the executor, which is part of the address to actors and used
    // for routing decisions
    name: String,

    // map of actor names to actors where the key (actor name) is part of the address-to-actor
    // resolution
    actors: HashMap<String, Box<dyn Actor>>,

    // Handle for sending commands to the current executor. Useful for spawning new actors
    // on the main event loop, or any action that may need to be performed in a slightly
    // delayed manner.
    sender: Sender<ExecutorCommands>,
}
impl ThreadExecutor {
    /// Utility function for ensuring that the name of an actor is unique. Useful before
    /// inserting a new entry in the actor store (when creating actors).
    fn assert_name_unique(&self, name: &str) {
        if self.actors.contains_key(name) {
            panic!("Actor name {} already exists", name);
        }
    }
}
impl Executor for ThreadExecutor {
    fn run(mut self, receiver: Receiver<ExecutorCommands>) {
        const SLEEP_DURATION_MS: u64 = 1;

        loop {
            if !receiver.is_empty() {
                match receiver.recv().unwrap() {
                    ExecutorCommands::AssignActor(mut actor, name) => {
                        debug!("received assign-root-actor command for actor {}", name);
                        self.assert_name_unique(&name);
                        trace!("calling before_start for actor {}", name);
                        actor.before_start(Context::new(&mut self));
                        self.actors.insert(name.clone(), actor);
                    }
                    ExecutorCommands::Shutdown => {
                        debug!("received shutdown command");
                        break;
                    }
                }
                // TODO: process messages
            }
            trace!("nothing to do, sleeping...");
            thread::sleep(Duration::from_millis(SLEEP_DURATION_MS));
        }
    }

    fn get_address(&self, actor_name: &str) -> ActorAddress {
        ActorAddress {
            name: actor_name.to_string(),
            executor_name: self.name.clone(),
        }
    }

    fn assign_actor(&self, actor: Box<dyn Actor>, name: String) {
        // TODO: this should be a non-blocking send to avoid dead-locking on a full
        //       command-queue.
        // However, need to think about how to allow for many actors to be created and used
        // within a single actor loop. :thinkies:
        // XXX: For now, this means that creating + 'asking' an actor would be a deadlock
        self.sender
            .send(ExecutorCommands::AssignActor(actor, name))
            .unwrap();
    }
}
