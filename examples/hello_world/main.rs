extern crate busan;

use busan::actor::{Actor, ActorInit};
use busan::config::ActorSystemConfig;
use busan::system::ActorSystem;
use std::thread;

pub mod hello_world {
    include!(concat!(env!("OUT_DIR"), "/hello_world.rs"));
}

fn main() {
    let mut system = ActorSystem::init(ActorSystemConfig::default());
    let init = hello_world::actor::Init {
        greeting: "Hi there!".to_string(),
    };
    system.spawn_root_actor::<_, Greet>("greeter".to_string(), &init);

    thread::sleep(std::time::Duration::from_secs(1));
    system.shutdown();
}

struct Greet {
    greeting: String,
}

impl ActorInit for Greet {
    type Init = hello_world::actor::Init;

    fn init(init_msg: &Self::Init) -> Self {
        println!("spawning greet actor");
        Greet {
            greeting: init_msg.greeting.clone(),
        }
    }
}

impl Actor for Greet {
    fn before_start(&mut self, _ctx: busan::actor::Context) {
        println!("{}", self.greeting);
    }
}
