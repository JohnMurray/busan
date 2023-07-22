use log::{debug, info, trace};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

use crate::actor::{cell_state, ActorCell, Context, SenderType, Uri};
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
    actor_cells: HashMap<Uri, ActorCell>,

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
            actor_cells: HashMap::new(),
            command_channel,
            runtime_manager,
        }
    }

    /// Utility function for ensuring that the address of an actor is unique. Useful before
    /// inserting a new entry in the actor store (when creating actors).
    fn assert_unique_address(&self, address: &Uri) {
        if self.actor_cells.contains_key(address) {
            panic!("Actor name {} already exists", address);
        }
    }
}

// Macro for quickly constructing a context object within the thread executor. The construction
// of the context almost always looks the same, just some slight differences with the sender.
macro_rules! context {
    ($self:tt, $cell:tt, $sender:path) => {
        context!($self, $cell, ($sender))
    };
    ($self:tt, $cell:tt, $sender:expr) => {
        Context {
            address: &$cell.address,
            runtime_manager: &$self.runtime_manager,
            executor_command_channel: &$self.command_channel,
            parent: &$cell.parent,
            children: &mut $cell.children,
            sender: &$sender,
            cell_state: &mut $cell.state,
        }
    };
}

impl Executor for ThreadExecutor {
    fn run(mut self) {
        const SLEEP_DURATION_MS: u64 = 1;

        loop {
            if !self.command_channel.recv_is_empty() {
                match self.command_channel.recv().unwrap() {
                    ExecutorCommands::AssignActor(mut cell) => {
                        debug!("received actor assignment for {}", &cell.address.uri);
                        self.assert_unique_address(&cell.address.uri);
                        trace!("calling before_start for actor {}", &cell.address.uri);
                        cell.actor
                            .before_start(context!(self, cell, SenderType::System));
                        self.actor_cells.insert(cell.address.uri.clone(), cell);
                    }
                    ExecutorCommands::ShutdownActor(address) => {
                        let cell = self.actor_cells.get_mut(&address.uri).unwrap();
                        cell_state::set_shutdown(&mut cell.state);
                        cell.actor
                            .before_stop(context!(self, cell, SenderType::System));

                        // Inform the runtime manager of the shutdown and also forward shutdown
                        // signals to the children.
                        self.runtime_manager.shutdown_actor(&address, false);
                        for child in &cell.children {
                            self.runtime_manager.shutdown_actor(child, true);
                        }

                        // TODO: Notify the parent and all of the watchers
                        //       thought: maybe this should be done by the runtime manager when the
                        //       shutdown signal is sent. (add an additional flag of "notify_parent"
                        //       to the message we send in `runtime_manager.shutdown_actor`).
                    }
                    ExecutorCommands::Shutdown => {
                        info!("received shutdown command");
                        break;
                    }
                }
            }
            // Iterate over the actor-cells and check if there are any non-empty mailboxes.
            // If one is found, process a message from it.
            let mut messages_processed = 0;
            for (_, cell) in self.actor_cells.iter_mut() {
                if !cell_state::is_shutdown(cell.state) {
                    // TODO: Forward messages to dead-letter-queue
                    continue;
                }
                if !cell.mailbox.is_empty() {
                    messages_processed += 1;
                    let result = cell.mailbox.try_recv();
                    if let Ok(letter) = result {
                        trace!("[{}] processing message: {:?}", &cell.address, &letter);
                        cell.actor
                            .receive(context!(self, cell, letter.sender), letter.payload);
                    }
                }
            }

            // If no messages were processed, sleep for a bit to avoid busy-waiting
            if messages_processed == 0 {
                trace!("nothing to do, sleeping...");
                thread::sleep(Duration::from_millis(SLEEP_DURATION_MS));
            }
        }

        self.runtime_manager.notify_shutdown(self.name);
    }
}
