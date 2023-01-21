use log::debug;
use std::collections::HashMap;
use std::thread;

use crate::actor::{Actor, ActorInit};
use crate::executor::thread_executor::ThreadExecutorFactory;
use crate::executor::{CommandChannel, ExecutorCommands, ExecutorFactory, ExecutorHandle};

const NUM_EXECUTORS: usize = 4;

/// Global value for a collection of actors that may communicate with local-only addresses.
pub struct ActorSystem {
    executor_factory: Box<dyn ExecutorFactory>,
    executors: HashMap<String, ExecutorHandle>,
    runtime_manager: RuntimeManagerRef,
    runtime_thread_handle: thread::JoinHandle<()>,
    root_actor_assigned: bool,
}

impl ActorSystem {
    pub fn init() -> ActorSystem {
        let mut runtime_manager = RuntimeManager::init();
        let executor_factory = Box::new(ThreadExecutorFactory {});
        let mut executors = HashMap::new();

        // create a pre-configured number of executors
        for i in 0..NUM_EXECUTORS {
            let command_channel = CommandChannel::new();
            let executor_name = format!("executor-{}", i);
            let executor_handle = executor_factory.spawn_executor(
                executor_name.clone(),
                command_channel.clone(),
                runtime_manager.get_ref(),
            );

            executors.insert(executor_name.clone(), executor_handle);
            runtime_manager.add_executor(executor_name.clone(), command_channel);
        }

        let runtime_manager_ref = runtime_manager.get_ref();
        let runtime_thread_handle = thread::spawn(move || {
            runtime_manager.run();
        });

        ActorSystem {
            executor_factory,
            executors,
            runtime_manager: runtime_manager_ref,
            runtime_thread_handle,
            root_actor_assigned: false,
        }
    }

    pub fn spawn_root_actor<B, A: ActorInit<Init = B> + Actor + 'static>(
        &mut self,
        name: String,
        init_msg: &B,
    ) {
        debug_assert!(
            self.executors.len() > 0,
            "No executors available to spawn actor"
        );
        debug_assert!(!self.root_actor_assigned, "Root actor already assigned");

        self.root_actor_assigned = true;
        self.runtime_manager
            .assign_actor(Box::new(A::init(init_msg)), name);
    }

    /// Send shutdown message to all executors and wait for them to finish
    pub fn shutdown(self) {
        self.runtime_manager.shutdown_system();
        self.await_shutdown();
    }

    /// Await shutdown of all executors. Similar to shutdown, but doesn't send
    /// shutdown messages to begin shutdown. Will wait indefinitely until all
    /// executors have shutdown.
    pub fn await_shutdown(self) {
        self.executors
            .into_iter()
            .for_each(|(_, manager)| manager.await_close());
        self.runtime_thread_handle.join().unwrap();
    }
}

struct RuntimeManager {
    /// Map of executor names to their command-channel (for sending commands)
    executor_command_channels: HashMap<String, CommandChannel<ExecutorCommands>>,

    manager_command_channel: CommandChannel<ManagerCommands>,

    round_robin_state: usize,
    shutdown_initiated: bool,
}

impl RuntimeManager {
    fn init() -> RuntimeManager {
        RuntimeManager {
            executor_command_channels: HashMap::new(),
            manager_command_channel: CommandChannel::new(),
            round_robin_state: 0,
            shutdown_initiated: false,
        }
    }

    fn add_executor(&mut self, name: String, command_channel: CommandChannel<ExecutorCommands>) {
        self.executor_command_channels.insert(name, command_channel);
    }

    fn get_ref(&self) -> RuntimeManagerRef {
        RuntimeManagerRef::new(self.manager_command_channel.clone())
    }

    fn run(mut self) {
        loop {
            match self.manager_command_channel.recv() {
                Ok(ManagerCommands::Shutdown) => {
                    if self.shutdown_initiated {
                        continue;
                    }
                    self.shutdown_initiated = true;
                    self.executor_command_channels
                        .iter()
                        .for_each(|(_, channel)| {
                            channel.send(ExecutorCommands::Shutdown).unwrap();
                        });
                }

                // TODO: Actually send this message from the Executors
                Ok(ManagerCommands::ExecutorShutdown { name }) => {
                    if self.executor_command_channels.contains_key(&name) {
                        self.executor_command_channels.remove(&name);
                        if self.executor_command_channels.len() == 0 {
                            break;
                        }
                    }
                }
                Ok(ManagerCommands::AssignActor(actor, name)) => {
                    let executor_name = self.get_next_executor();
                    self.executor_command_channels
                        .get(&executor_name)
                        .unwrap()
                        .send(ExecutorCommands::AssignActor(actor, name))
                        .unwrap();
                }
                Err(_) => {}
            }
        }

        debug!("Runtime manager shutting down");
    }

    fn get_next_executor(&mut self) -> String {
        let mut iter = self.executor_command_channels.iter();
        if self.rohnd_robin_state >= iter.len() {
            self.round_robin_state = 0;
        }
        iter.nth(self.round_robin_state).unwrap().0.clone()
    }
}

pub struct RuntimeManagerRef {
    manager_command_channel: CommandChannel<ManagerCommands>,
}

impl RuntimeManagerRef {
    fn new(manager_command_channel: CommandChannel<ManagerCommands>) -> RuntimeManagerRef {
        RuntimeManagerRef {
            manager_command_channel,
        }
    }

    pub fn shutdown_system(&self) {
        self.manager_command_channel
            .send(ManagerCommands::Shutdown)
            .unwrap();
    }

    pub fn notify_shutdown(&self, executor_name: String) {
        self.manager_command_channel
            .send(ManagerCommands::ExecutorShutdown {
                name: executor_name,
            })
            .unwrap();
    }

    pub fn assign_actor<A: Actor + 'static>(&self, actor: Box<A>, name: String) {
        self.manager_command_channel
            .send(ManagerCommands::AssignActor(actor, name))
            .unwrap();
    }
}

enum ManagerCommands {
    Shutdown,
    ExecutorShutdown { name: String },
    AssignActor(Box<dyn Actor>, String),
}
