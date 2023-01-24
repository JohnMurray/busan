use num_cpus;

/// Configuration struct for an ActorSystem.
pub struct ActorSystemConfig {
    pub executor_config: ExecutorConfig,
}

impl ActorSystemConfig {
    pub fn validate(&self) -> Result<(), String> {
        self.executor_config.validate()
    }
}

impl Default for ActorSystemConfig {
    fn default() -> Self {
        ActorSystemConfig {
            executor_config: ExecutorConfig::default(),
        }
    }
}

pub struct ExecutorConfig {
    /// The number of executors to spawn
    pub num_executors: usize,

    // The type of executor to use
    pub executor_type: ExecutorType,
}

pub enum ExecutorType {
    Thread,
}

impl ExecutorConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.num_executors == 0 {
            return Err("num_executors must be greater than 0".to_string());
        }

        Ok(())
    }
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        ExecutorConfig {
            num_executors: num_cpus::get(),
            executor_type: ExecutorType::Thread,
        }
    }
}
