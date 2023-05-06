use crossbeam_channel::{bounded, unbounded, Sender};
use log::{info, warn};
use std::collections::HashMap;
use std::thread;

use crate::actor::{Actor, ActorAddress, ActorCell, ActorInit, Letter, Uri};
use crate::executor::{get_executor_factory, ExecutorCommands, ExecutorHandle};
use crate::util::CommandChannel;
use crate::{actor, config};

/// `ActorSystem`is the top-level handle for an actor system.
///
/// It is both the entrypoint for creating a new actor system (via [`init`](ActorSystem::init) and
/// [`spawn_root_actor`](ActorSystem::spawn_root_actor)) and also the handle used to wait for the
/// system to complete or to initiate a shutdown ([`await_shutdown`](ActorSystem::await_shutdown)
/// and [`shutdown`](ActorSystem::shutdown), respectively).
///
/// Once the ActorSystem is initialized through [`init`](ActorSystem::init), control and management
/// of the executors and actors is delegated to a runtime management thread.
pub struct ActorSystem {
    executors: HashMap<String, ExecutorHandle>,
    runtime_manager: RuntimeManagerRef,
    runtime_thread_handle: thread::JoinHandle<()>,
    root_actor_assigned: bool,
}

impl ActorSystem {
    /// Initialize (and start) a new actor system. This will start up the runtime management thread
    /// and spawn executors for running actor-related actions. The nature and number of executors
    /// and other behaviors will be determined by the provided `config`.
    ///
    /// ```rust
    /// # use busan::message::common_types::StringWrapper;
    /// # use busan::message::ToMessage;
    /// # use busan::prelude::*;
    /// # struct GreetActor{value: String}
    /// # impl Actor for GreetActor {
    /// #     fn before_start(&mut self, _ctx: Context) { println!("Hello {}", self.value); }
    /// #     fn receive(&mut self, _ctx: Context, _msg: Box<dyn Message>) { println!("Hello {}", self.value); }
    /// # }
    /// # impl ActorInit for GreetActor{
    /// #     type Init = StringWrapper;
    /// #     fn init(init_msg: &StringWrapper) -> Self { GreetActor{ value: init_msg.value.clone()} }
    /// # }
    /// fn main() {
    ///   let mut system = ActorSystem::init(ActorSystemConfig::default());
    ///   system.spawn_root_actor::<_, GreetActor>("greet-actor", &"World".to_message());
    ///   system.shutdown();
    /// }
    /// ```
    pub fn init(config: config::ActorSystemConfig) -> ActorSystem {
        config.validate().unwrap();

        let mut runtime_manager = RuntimeManager::init();
        let executor_factory = get_executor_factory(&config.executor_config.executor_type);
        let mut executors = HashMap::new();

        // create a pre-configured number of executors
        for i in 0..(config.executor_config.num_executors) {
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
            executors,
            runtime_manager: runtime_manager_ref,
            runtime_thread_handle,
            root_actor_assigned: false,
        }
    }

    /// Spawn the root actor for the system. The root actor will be the actor at the top
    /// of the actor hierarchy and all other actors must be created from here. Note that
    /// there may only be a single root actor per system and can, in some ways, be considered
    /// the "main" function of the actor system.
    pub fn spawn_root_actor<B, A: ActorInit<Init = B> + Actor + 'static>(
        &mut self,
        name: &str,
        init_msg: &B,
    ) {
        debug_assert!(
            !self.executors.is_empty(),
            "No executors available to spawn actor"
        );
        debug_assert!(!self.root_actor_assigned, "Root actor already assigned");

        self.root_actor_assigned = true;
        self.runtime_manager.assign_actor(
            Box::new(A::init(init_msg)),
            ActorAddress::new_root(name),
            None,
        );
    }

    /// Send shutdown message to all executors and wait for them to finish. This includes
    /// waiting for the runtime manager to shutdown as well.
    pub fn shutdown(self) {
        self.runtime_manager.shutdown_system();
        self.await_shutdown();
    }

    /// Await shutdown of all executors. Similar to shutdown, but doesn't send
    /// shutdown messages to begin shutdown. Will wait indefinitely until all
    /// executors and the runtime manager have shutdown.
    pub fn await_shutdown(self) {
        self.executors
            .into_iter()
            .for_each(|(_, manager)| manager.await_close());
        self.runtime_thread_handle.join().unwrap();
    }
}

/// `RuntimeManager` is an administrative process that manages the executors and runs in
/// it's own thread. It is responsible for coordinating amongst executors as well as
/// proxying commands from the `ActorSystem`.
///
/// For a full list of commands, see the `RuntimeManagerCommand` enum.
///
/// For the public API of the `RuntimeManager`, see the `RuntimeManagerRef` struct.
struct RuntimeManager {
    /// Map of executor names to their command-channel (for sending commands)
    executor_command_channels: HashMap<String, CommandChannel<ExecutorCommands>>,
    actor_registry: HashMap<Uri, actor::Mailbox>,

    manager_command_channel: CommandChannel<ManagerCommands>,

    round_robin_state: usize,
    shutdown_initiated: bool,
}

impl RuntimeManager {
    fn init() -> RuntimeManager {
        RuntimeManager {
            executor_command_channels: HashMap::new(),
            actor_registry: HashMap::new(),
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

                Ok(ManagerCommands::ExecutorShutdown { name }) => {
                    if self.executor_command_channels.contains_key(&name) {
                        self.executor_command_channels.remove(&name);
                        if self.executor_command_channels.is_empty() {
                            break;
                        }
                    }
                }
                Ok(ManagerCommands::AssignActor {
                    actor,
                    address,
                    parent,
                }) => {
                    let executor_name = self.get_next_executor();
                    let (sender, receiver) = unbounded::<Letter>();
                    let address_uri = address.uri.clone();
                    address.set_mailbox(sender.clone());
                    let cell = ActorCell::new(actor, receiver, address, parent);

                    self.actor_registry.insert(address_uri, sender);

                    self.executor_command_channels
                        .get(&executor_name)
                        .unwrap()
                        .send(ExecutorCommands::AssignActor(cell))
                        .unwrap();
                }
                Ok(ManagerCommands::ResolveAddress {
                    address_uri,
                    return_channel,
                }) => {
                    let mailbox_lookup = self.actor_registry.get(&address_uri);
                    let result = match mailbox_lookup {
                        Some(mailbox) => return_channel.try_send(Some(mailbox.clone())),
                        None => return_channel.try_send(None),
                    };
                    if result.is_err() {
                        warn!(
                            "Failed to send address resolution result on return channel: {}",
                            result.err().unwrap(),
                        );
                    }
                }
                Err(_) => {}
            }
        }

        info!("Runtime manager shutting down");
    }

    fn get_next_executor(&mut self) -> String {
        let mut iter = self.executor_command_channels.iter();
        if self.round_robin_state >= iter.len() {
            self.round_robin_state = 0;
        }
        iter.nth(self.round_robin_state).unwrap().0.clone()
    }
}

/// `RuntimeManagerRef` is a handle for communicating to the runtime manager in a thread-safe
/// manner. The `RuntimeManager` may only be interacted with through the `RuntimeManagerRef`.
pub struct RuntimeManagerRef {
    manager_command_channel: CommandChannel<ManagerCommands>,
}

impl RuntimeManagerRef {
    fn new(manager_command_channel: CommandChannel<ManagerCommands>) -> RuntimeManagerRef {
        RuntimeManagerRef {
            manager_command_channel,
        }
    }

    /// Signal to the runtime manager to begin shutting down the system. This will result in
    /// shutdown notifications being sent to all of the executors.
    pub(crate) fn shutdown_system(&self) {
        self.manager_command_channel
            .send(ManagerCommands::Shutdown)
            .unwrap();
    }

    /// Signal to the runtime manager the the executor has completed (or is very near completing)
    /// shutdown. This should only be called by the executor itself as the final step of it's
    /// shutdown process.
    pub(crate) fn notify_shutdown(&self, executor_name: String) {
        self.manager_command_channel
            .send(ManagerCommands::ExecutorShutdown {
                name: executor_name,
            })
            .unwrap();
    }

    /// Request that a new actor be assigned to a runtime executor. This may be called when assigning
    /// either a root actor or a child actor. This should be used to avoid blocking actor creation
    /// on a single executor.
    pub(crate) fn assign_actor(
        &self,
        actor: Box<dyn Actor>,
        address: ActorAddress,
        parent: Option<ActorAddress>,
    ) {
        self.manager_command_channel
            .send(ManagerCommands::AssignActor {
                actor,
                address,
                parent,
            })
            .unwrap();
    }

    /// Resolve an address to mailbox by looking up the actor in the global registry. Note that this
    /// will block until the management thread has performed the lookup.
    pub(crate) fn resolve_address(&self, address: &ActorAddress) -> Option<actor::Mailbox> {
        let uri = address.uri.clone();
        let (sender, receiver) = bounded::<Option<actor::Mailbox>>(1);
        self.manager_command_channel
            .send(ManagerCommands::ResolveAddress {
                address_uri: uri,
                return_channel: sender,
            })
            .unwrap();

        receiver.recv().unwrap()
    }
}

/// `ManagerCommands` is the set of commands that the `RuntimeManager` can receive from the
/// `RuntimeRef`. While this is purely an internal struct, it can be useful in understanding
/// the behaviors of the `RuntimeManager`.
enum ManagerCommands {
    /// Request that the actor system shutdown
    Shutdown,

    /// Notification from an executor (identified by the name field) that it has completed shutdown
    ExecutorShutdown { name: String },

    /// A request that a newly constructed `Actor` be "realized" in the actor system.
    ///   + Wrap the actor into an `ActorCell` with a mailbox and address
    ///   + Store the `ActorAddress` in a global registry for address resolution
    ///   + Assign the actor to an executor
    AssignActor {
        actor: Box<dyn Actor>,
        address: ActorAddress,
        parent: Option<ActorAddress>,
    },

    /// A request to resolve an actor address to a mailbox. This is given a direct return
    /// channel so the sender can block on the result of the lookup if desired.
    ResolveAddress {
        address_uri: Uri,
        return_channel: Sender<Option<actor::Mailbox>>,
    },
}
