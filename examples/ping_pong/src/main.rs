#![allow(non_snake_case)]
use busan::actor::{Actor, ActorInit, Context};
use busan::config::ActorSystemConfig;
use busan::message::common_types::{I32Wrapper, StringWrapper};
use busan::message::{Message, ToMessage};
use busan::system::ActorSystem;
use std::thread;

struct Ping {}
struct Pong {}

impl ActorInit for Ping {
    type Init = I32Wrapper;

    fn init(_init_msg: &Self::Init) -> Self
    where
        Self: Sized + Actor,
    {
        println!("init ping");
        Ping {}
    }
}

impl ActorInit for Pong {
    type Init = I32Wrapper;

    fn init(_init_msg: &Self::Init) -> Self
    where
        Self: Sized + Actor,
    {
        println!("init pong");
        Pong {}
    }
}

impl Actor for Ping {
    fn before_start(&mut self, mut ctx: Context) {
        let pong_addr =
            Some(ctx.spawn_child::<_, Pong>("pong".to_string(), &I32Wrapper::default()));
        ctx.send_message(pong_addr.as_ref().unwrap(), "ping".to_message());
    }

    fn receive(&mut self, ctx: Context, msg: Box<dyn Message>) {
        // Print the message and respond with a "ping"
        if let Some(strMsg) = msg.as_any().downcast_ref::<StringWrapper>() {
            println!("received message: {}", strMsg.value);
            ctx.send_message(ctx.sender(), "ping".to_message());
        }
    }
}
impl Actor for Pong {
    fn receive(&mut self, ctx: Context, msg: Box<dyn Message>) {
        // Print the message and respond with a "pong"
        if let Some(strMsg) = msg.as_any().downcast_ref::<StringWrapper>() {
            println!("received message: {}", strMsg.value);
            ctx.send_message(ctx.sender(), "pong".to_message());
        }
    }
}

fn main() {
    env_logger::Builder::new()
        .filter_level(::log::LevelFilter::Debug)
        .init();

    let mut system = ActorSystem::init(ActorSystemConfig::default());
    system.spawn_root_actor::<_, Ping>("ping".to_string(), &I32Wrapper::default());

    thread::sleep(std::time::Duration::from_secs(1));
    system.shutdown();
}
