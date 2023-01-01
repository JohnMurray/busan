pub(crate) mod thread_executor;

use crossbeam_channel::{Receiver, Sender};

use crate::actor::Actor;

pub enum ExecutorCommands {
    SpawnActor(Box<dyn Actor>, String),
    Shutdown,
}

/// responsible for creating an executor
pub trait ExecutorFactory {
    // Spawn an executor with a given name. Tha name will be used by the
    // executor for routing messages to the correct actor.
    fn spawn_executor(&self, name: String) -> ExecutorHandle;
}

pub trait Executor {
    fn run(&mut self, receiver: Receiver<ExecutorCommands>);
}

/// ExecutorHandle contains all the context necessary for the control-thread, which
/// amounts to a channel to send commands through and a way to close or await closing
/// of the executor.
pub struct ExecutorHandle {
    pub sender: Sender<ExecutorCommands>,
    close_fn: Box<dyn FnOnce() -> ()>,
}

impl ExecutorHandle {
    pub fn new<F: FnOnce() -> () + 'static>(
        sender: Sender<ExecutorCommands>,
        close_fn: F,
    ) -> ExecutorHandle {
        ExecutorHandle {
            sender,
            close_fn: Box::new(close_fn),
        }
    }

    /// Close the executor handle. Note that this can only be called once and consumes itself.
    pub fn await_close(self) {
        (self.close_fn)();
    }
}
