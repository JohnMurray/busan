use log::{debug, info, trace};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

use crate::actor::{ActorCell, Context, Letter, SenderType, Uri};
use crate::executor::{
    CommandChannel, Executor, ExecutorCommands, ExecutorFactory, ExecutorHandle,
};
use crate::message::system::PoisonPill;
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
            executor_channel: &$self.command_channel,
            parent: &$cell.parent,
            children: &mut $cell.children,
            sender: &$sender,
        }
    };
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

    /// Process a system message for a given actor.
    ///
    /// System messages have special handling logic that that involves the executor, so they
    /// can't be passed on to the actor in the traditional fashion. Depending on the message,
    /// they still may be forwarded to the actor.
    fn process_system_message(letter: Letter, cell: &mut ActorCell, context: Context) {
        if let Some(_) = letter.payload.as_any().downcast_ref::<PoisonPill>() {
            trace!(
                "received poison pill for {}. Calling shutdown hook",
                &cell.address.uri
            );
            cell.actor.before_shutdown(context);
            // TODO: Signal shutdown to the execute
        } else {
            // System messages should always be known as only messages defined by the crate
            // can be system messages *and* system messages cannot be sent remotely.
            panic!("Unknown system message type: {:?}", letter.payload);
        }
    }

    fn shutdown_actor(&mut self, uri: &Uri) {
        if let Some(mut cell) = self.actor_cells.remove(&uri) {
            trace!("calling before_shutdown for actor {}", &uri);
            // TODO: Send the remaining messages in the mailbox to the dead letter queue
            // TODO: Figure out how to redirect all Sender<Letter> handles to the dead letter queue
            cell.actor
                .before_shutdown(context!(self, cell, SenderType::System));
        }
    }

    fn assign_actor(&mut self, mut cell: ActorCell) {
        debug!("received actor assignment for {}", &cell.address.uri);
        self.assert_unique_address(&cell.address.uri);
        trace!("calling before_start for actor {}", &cell.address.uri);
        cell.actor
            .before_start(context!(self, cell, SenderType::System));
        self.actor_cells.insert(cell.address.uri.clone(), cell);
    }
}

impl Executor for ThreadExecutor {
    fn run(mut self) {
        const SLEEP_DURATION_MS: u64 = 1;

        loop {
            // Handle executor commands before handling any actor messages. Generally it is expected
            // to have very few of these per loop.
            if !self.command_channel.recv_is_empty() {
                match self.command_channel.recv().unwrap() {
                    ExecutorCommands::AssignActor(cell) => {
                        self.assign_actor(cell);
                    }
                    ExecutorCommands::ShutdownActor(address) => {
                        self.shutdown_actor(&address.uri);
                    }
                    ExecutorCommands::Shutdown => {
                        info!("received shutdown command");
                        // Break to exit the main 'loop' in the run function
                        break;
                    }
                }
            }
            let mut messages_processed = 0;
            // Iterate over the actor-cells and check if there are any non-empty mailboxes.
            // If one is found, process a single message from it. This maintains fairness
            // amongst message processing, but may result in large amounts of waste if there
            // are a high number of mostly idle actors.
            let cells_iter = self.actor_cells.iter_mut();
            for (_, cell) in cells_iter {
                if !cell.mailbox.is_empty() {
                    let result = cell.mailbox.try_recv();
                    if let Ok(letter) = result {
                        messages_processed += 1;
                        // If the letter is a system message, perform any necessary pre-processing
                        // of the message before (potentially) passing it on to the actor.
                        if letter
                            .payload
                            .is_system_message(&crate::message::private::Local::Value)
                        {
                            trace!(
                                "[{}] processing system message: {:?}",
                                &cell.address,
                                &letter
                            );
                            // self.process_system_message(letter, cell);
                            // TODO: Mark the cell as unable to receive/process new messages (???)
                            // TODO: Send shutdown signals to all children
                            // TODO: Wait for exit signals from children
                        }
                        // For all non-system messages, pass the message directly to the actor
                        else {
                            trace!("[{}] processing message: {:?}", &cell.address, &letter);
                            cell.actor
                                .receive(context!(self, cell, letter.sender), letter.payload);
                        }
                    }
                }
            }

            // Inject a small sleep in the thread executor if a loop resulted in no work being
            // performed. Otherwise the executor will spin at 100% CPU usage.
            if (messages_processed == 0) {
                trace!("nothing to do, sleeping...");
                thread::sleep(Duration::from_millis(SLEEP_DURATION_MS));
            }
        }

        self.runtime_manager.notify_shutdown(self.name);
    }
}
