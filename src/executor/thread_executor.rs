use log::{debug, trace};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

use crate::actor::{Actor, ActorAddress, Context};
use crate::executor::{
    CommandChannel, Executor, ExecutorCommands, ExecutorFactory, ExecutorHandle,
};
use crate::system::RuntimeManagerRef;

pub struct ThreadExecutorFactory {}
impl ExecutorFactory for ThreadExecutorFactory {
    fn spawn_executor(
        &self,
        name: String,
        command_channel: CommandChannel<ExecutorCommands>,
        manager_ref: RuntimeManagerRef,
    ) -> ExecutorHandle {
        let t =
            thread::spawn(move || ThreadExecutor::init(name, command_channel, manager_ref).run());
        ExecutorHandle::new(move || t.join().unwrap())
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
    command_channel: CommandChannel<ExecutorCommands>,

    // Handle for sending message to the manager. This is useful for coordinating system-wide
    // actions such as shutting down the system, spawning new actors, etc.
    runtime_manager: RuntimeManagerRef,
}
impl ThreadExecutor {
    fn init(
        name: String,
        command_channel: CommandChannel<ExecutorCommands>,
        runtime_manager: RuntimeManagerRef,
    ) -> ThreadExecutor {
        ThreadExecutor {
            name,
            actors: HashMap::new(),
            command_channel,
            runtime_manager,
        }
    }

    /// Utility function for ensuring that the name of an actor is unique. Useful before
    /// inserting a new entry in the actor store (when creating actors).
    fn assert_name_unique(&self, name: &str) {
        if self.actors.contains_key(name) {
            panic!("Actor name {} already exists", name);
        }
    }
}
impl Executor for ThreadExecutor {
    fn run(mut self) {
        const SLEEP_DURATION_MS: u64 = 1;

        loop {
            if !self.command_channel.recv_is_empty() {
                match self.command_channel.recv().unwrap() {
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

        self.runtime_manager.notify_shutdown(self.name);
    }

    fn get_address(&self, actor_name: &str) -> ActorAddress {
        ActorAddress {
            name: actor_name.to_string(),
            executor_name: self.name.clone(),
        }
    }

    fn assign_actor(&self, actor: Box<dyn Actor>, name: String) {
        self.runtime_manager.assign_actor(actor, name);
    }
}
