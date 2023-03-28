#![allow(non_snake_case)]
use busan::actor::{Actor, ActorAddress, ActorInit, Context};
use busan::config::ActorSystemConfig;
use busan::message::common_types::{I32Wrapper, StringWrapper};
use busan::message::{Message, ToMessage};
use busan::system::ActorSystem;
use std::thread;

struct Ping {
    pong_addr: Option<ActorAddress>,
}
struct Pong {
    ping_addr: Option<ActorAddress>,
}

impl ActorInit for Ping {
    type Init = I32Wrapper;

    fn init(_init_msg: &Self::Init) -> Self
    where
        Self: Sized + Actor,
    {
        println!("init ping");
        Ping { pong_addr: None }
    }
}

impl ActorInit for Pong {
    type Init = I32Wrapper;

    fn init(_init_msg: &Self::Init) -> Self
    where
        Self: Sized + Actor,
    {
        println!("init pong");
        Pong { ping_addr: None }
    }
}

impl Actor for Ping {
    fn before_start(&mut self, mut ctx: Context) {
        self.pong_addr =
            Some(ctx.spawn_child::<_, Pong>("pong".to_string(), &I32Wrapper::default()));
        ctx.send_message(self.pong_addr.as_ref().unwrap(), "ping".to_message());
    }

    fn receive(&mut self, ctx: Context, _msg: Box<dyn Message>) {
        println!("received message");
        // assume it was a pong, send a ping
        match &self.pong_addr {
            Some(addr) => ctx.send_message(addr, "ping".to_message()),
            None => {}
        }
    }
}
impl Actor for Pong {
    fn receive(&mut self, ctx: Context, msg: Box<dyn Message>) {
        println!("received message");
        // assume it was a ping, send a pong
        if let Some(strMsg) = msg.as_any().downcast_ref::<StringWrapper>() {
            println!("received message: {}", strMsg.value);
        }
        if let Some(addr) = &self.ping_addr {
            ctx.send_message(addr, "pong".to_message());
        }
    }
}

fn main() {
    let mut system = ActorSystem::init(ActorSystemConfig::default());
    system.spawn_root_actor::<_, Ping>("ping".to_string(), &I32Wrapper::default());

    thread::sleep(std::time::Duration::from_secs(1));
    system.shutdown();
}
