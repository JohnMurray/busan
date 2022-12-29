use std::collections::HashMap;
use std::thread;
use std::time::Duration;

use crate::actor::{Actor, ActorInit};
use crate::executor::thread_executor::ThreadExecutorFactory;
use crate::executor::{ExecutorCommands, ExecutorFactory, ExecutorHandle};

const NUM_EXECUTORS: usize = 4;

/// Global value for a collection of actors that may communicate with local-only addresses.
pub struct ActorSystem {
    executor_factory: Box<dyn ExecutorFactory>,
    executors: HashMap<String, ExecutorHandle>,
}

impl ActorSystem {
    pub fn init() -> ActorSystem {
        // create a mutable actor system with a thread executor factory
        let mut system = ActorSystem {
            executor_factory: Box::new(ThreadExecutorFactory {}),
            executors: HashMap::new(),
        };

        // create a pre-configured number of executors
        for i in 0..NUM_EXECUTORS {
            system.executors.insert(
                format!("executor-{}", i),
                system
                    .executor_factory
                    .spawn_executor(format!("executor-{}", i)),
            );
        }

        system
    }

    // TODO: rename to root-actor only (or update to rotate through executors)
    // TODO: If for root actor only, create lock to prevent multiple root actors
    pub fn spawn_root_actor<B, A: ActorInit<Init = B> + Actor + 'static>(
        &self,
        name: String,
        init_msg: &B,
    ) {
        debug_assert!(
            self.executors.len() > 0,
            "No executors available to spawn actor"
        );

        self.executors
            .get("executor-0")
            .unwrap()
            .sender
            .send(ExecutorCommands::SpawnActor(
                Box::new(A::init(init_msg)),
                name,
            ))
            .unwrap()
    }

    /// Send shutdown message to all executors and wait for them to finish
    pub fn shutdown(self) {
        for (_, executor) in self.executors.iter() {
            executor.sender.send(ExecutorCommands::Shutdown).unwrap();
        }
        self.await_shutdown();
    }

    /// Await shutdown of all executors. Similar to shutdown, but doesn't send
    /// shutdown messages to begin shutdown. Will wait indefinitely until all
    /// executors have shutdown.
    pub fn await_shutdown(self) {
        self.executors
            .into_iter()
            .for_each(|(_, executor)| executor.await_close());
    }
}
