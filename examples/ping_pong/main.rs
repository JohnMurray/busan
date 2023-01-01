extern crate busan;

use busan::actor::{Actor, ActorInit, Context};
use busan::system::ActorSystem;

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
    let system = ActorSystem::init();
    system.spawn_root_actor::<_, Ping>("ping".to_string(), &());

    system.await_shutdown();
}
