extern crate busan;

use busan::actor::{Actor, ActorInit, Context};
use busan::config::ActorSystemConfig;
use busan::system::ActorSystem;
use std::thread;

struct Ping {}
struct Pong {}

impl ActorInit for Ping {
    type Init = ();

    fn init(_init_msg: &Self::Init) -> Self
    where
        Self: Sized + Actor,
    {
        println!("init ping");
        Ping {}
    }
}

impl ActorInit for Pong {
    type Init = ();

    fn init(_init_msg: &Self::Init) -> Self
    where
        Self: Sized + Actor,
    {
        println!("init pong");
        Pong {}
    }
}

impl Actor for Ping {
    fn before_start(&mut self, ctx: Context) {
        ctx.spawn_child::<_, Pong>("pong".to_string(), &());
    }
}
impl Actor for Pong {}

fn main() {
    let mut system = ActorSystem::init(ActorSystemConfig::default());
    system.spawn_root_actor::<_, Ping>("ping".to_string(), &());

    thread::sleep(std::time::Duration::from_secs(1));
    system.shutdown();
}
