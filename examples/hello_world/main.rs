extern crate busan;

use busan::actor::Actor;
use busan::system::ActorSystem;

fn main() {
    let system = ActorSystem::init();
    system.spawn_actor::<Greet>("greeter".to_string());

    system.wait_shutdown();
}

struct Greet {}
impl Actor for Greet {
    fn init() -> Self
    where
        Self: Sized,
    {
        println!("Hello, init()!");
        Greet {}
    }
}
