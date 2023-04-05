#![allow(non_snake_case)]
use busan::actor::{Actor, ActorInit};
use busan::config::ActorSystemConfig;
use busan::message::common_types::StringWrapper;
use busan::message::Message;
use busan::system::ActorSystem;
use std::thread;

mod proto {
    include!(concat!(env!("OUT_DIR"), "/hello_world.rs"));
}
use proto::*;

fn main() {
    let mut system = ActorSystem::init(ActorSystemConfig::default());
    let init = Init {
        greeting: "Hi there!".to_string(),
    };
    system.spawn_root_actor::<_, Greet>("greeter", &init);

    thread::sleep(std::time::Duration::from_secs(1));
    system.shutdown();
}

struct Greet {
    greeting: String,
}

impl ActorInit for Greet {
    type Init = Init;

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

    fn receive(&mut self, ctx: busan::actor::Context, msg: Box<dyn Message>) {
        match msg.as_any().downcast_ref::<StringWrapper>() {
            Some(msg) => println!("received message: {}", msg.value),
            None => self.unhandled(ctx, msg),
        }
    }
}
