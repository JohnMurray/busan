extern crate busan;

use busan::actor::{Actor, ActorInit};
use busan::system::ActorSystem;

pub mod hello_world {
    include!(concat!(env!("OUT_DIR"), "/hello_world.rs"));
}

fn main() {
    let system = ActorSystem::init();
    let mut init = hello_world::actor::Init::default();
    init.greeting = "Hi there!".to_string();
    system.spawn_root_actor::<_, Greet>("greeter".to_string(), &init);
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
    // fn init(init_msg: &dyn prost::Message) -> Self
    // where
    //     Self: Sized,
    // {
    //     if let &hello_world::actor::Init { greeting } = init_msg {
    //         println("{}", init_msg.name);
    //         return Greet {
    //             greeting: init_msg.name.clone(),
    //         };
    //     }
    //     Greet {
    //         greeting: "Hello, world".to_string(),
    //     }
    // }
}
