//! Runtime executor implementations for actors

pub(crate) mod thread_executor;

use crate::actor::{ActorAddress, ActorCell};
use crate::config::ExecutorType;
use crate::system::RuntimeManagerRef;
use crate::util::CommandChannel;

pub enum ExecutorCommands {
    AssignActor(ActorCell),
    ShutdownActor(ActorAddress),
    ShutdownActorComplete(ActorAddress),
    Shutdown,
}

/// responsible for creating an executor
pub trait ExecutorFactory {
    // Spawn an executor with a given name. Tha name will be used by the
    // executor for routing messages to the correct actor.
    fn spawn_executor(
        &self,
        name: String,
        command_channel: CommandChannel<ExecutorCommands>,
        manager_ref: RuntimeManagerRef,
    ) -> ExecutorHandle;
}

pub trait Executor {
    fn run(self);
}

/// ExecutorHandle contains all the context necessary for the control-thread, which
/// amounts to a channel to send commands through and a way to close or await closing
/// of the executor.
pub struct ExecutorHandle {
    close_fn: Box<dyn FnOnce()>,
}

impl ExecutorHandle {
    pub fn new<F: FnOnce() + 'static>(close_fn: F) -> ExecutorHandle {
        ExecutorHandle {
            close_fn: Box::new(close_fn),
        }
    }

    /// Close the executor handle. Note that this can only be called once and consumes itself.
    pub(crate) fn await_close(self) {
        (self.close_fn)();
    }
}

/// A static function that can be used to convert the config ExecutorType into a concrete
/// ExecutorFactory.
pub fn get_executor_factory(executor_type: &ExecutorType) -> Box<dyn ExecutorFactory> {
    match executor_type {
        ExecutorType::Thread => Box::new(thread_executor::ThreadExecutorFactory {}),
    }
}
