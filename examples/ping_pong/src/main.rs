use busan::actor::{Actor, ActorInit, Context};
use busan::config::ActorSystemConfig;
use busan::message::common_types::{I32Wrapper, StringWrapper};
use busan::message::Message;
use busan::system::ActorSystem;
use std::thread;

struct Ping {}
struct Pong {}

impl ActorInit for Ping {
    type Init = I32Wrapper;

    fn init(_init_msg: Self::Init) -> Self
    where
        Self: Sized + Actor,
    {
        println!("init ping");
        Ping {}
    }
}

impl ActorInit for Pong {
    type Init = I32Wrapper;

    fn init(_init_msg: Self::Init) -> Self
    where
        Self: Sized + Actor,
    {
        println!("init pong");
        Pong {}
    }
}

impl Actor for Ping {
    fn before_start(&mut self, mut ctx: Context) {
        let pong_addr = Some(ctx.spawn_child::<Pong, _, _>("pong", 0));
        ctx.send(pong_addr.as_ref().unwrap(), "ping");
    }

    fn receive(&mut self, ctx: Context, msg: Box<dyn Message>) {
        // Print the message and respond with a "ping"
        if let Some(str_msg) = msg.as_any().downcast_ref::<StringWrapper>() {
            println!("received message: {}", str_msg.value);
            ctx.send(ctx.sender(), "ping");
        }
    }
}
impl Actor for Pong {
    fn receive(&mut self, ctx: Context, msg: Box<dyn Message>) {
        // Print the message and respond with a "pong"
        if let Some(str_msg) = msg.as_any().downcast_ref::<StringWrapper>() {
            println!("received message: {}", str_msg.value);
            ctx.send(ctx.sender(), "pong");
        }
    }
}

fn main() {
    env_logger::Builder::new()
        .filter_level(::log::LevelFilter::Debug)
        .init();

    let mut system = ActorSystem::init(ActorSystemConfig::default());
    system.spawn_root_actor::<Ping, _, _>("ping", 0);

    thread::sleep(std::time::Duration::from_secs(1));
    system.shutdown();
}
