use busan::actor::{Actor, ActorAddress, ActorInit, Context};
use busan::config::ActorSystemConfig;
use busan::message::common_types::{I32Wrapper, U32Wrapper};
use busan::message::system::Ack;
use busan::message::Message;
use busan::system::ActorSystem;
use std::thread;

struct Distributor {
    worker: Option<ActorAddress>,
    work_ack_nonce: Option<u32>,
    work_queue: Vec<u32>,
}
struct Worker {}

impl ActorInit for Distributor {
    type Init = I32Wrapper;

    fn init(_init_msg: Self::Init) -> Self
    where
        Self: Sized + Actor,
    {
        println!("init distributor");
        Distributor {
            worker: None,
            work_ack_nonce: None,
            work_queue: vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
        }
    }
}

impl ActorInit for Worker {
    type Init = I32Wrapper;

    fn init(_init_msg: Self::Init) -> Self
    where
        Self: Sized + Actor,
    {
        println!("init worker");
        Worker {}
    }
}

impl Actor for Distributor {
    fn before_start(&mut self, mut ctx: Context) {
        self.worker = Some(ctx.spawn_child::<Worker, _, _>("worker", 0));
        self.send_work(&mut ctx);
    }

    fn receive(&mut self, mut ctx: Context, msg: Box<dyn Message>) {
        // If we receive an ack for work, send a message to do more work
        if let Some(ack) = msg.as_any().downcast_ref::<Ack>() {
            if self.work_ack_nonce.is_some() && ack.nonce == self.work_ack_nonce.unwrap() {
                self.send_work(&mut ctx);
            }
        }
    }
}

impl Distributor {
    fn send_work(&mut self, ctx: &mut Context) {
        if let Some(work) = self.work_queue.pop() {
            println!("pop'ing work: {}", work);
            self.work_ack_nonce = Some(ctx.send_with_ack(self.worker.as_ref().unwrap(), work));
        }
    }
}

impl Actor for Worker {
    fn receive(&mut self, _: Context, msg: Box<dyn Message>) {
        if let Some(work_msg) = msg.as_any().downcast_ref::<U32Wrapper>() {
            println!("received work: {}", work_msg.value);
        } else {
            println!("received unknown work");
        }
    }
}

fn main() {
    env_logger::Builder::new()
        .filter_level(::log::LevelFilter::Debug)
        .init();

    let mut system = ActorSystem::init(ActorSystemConfig::default());
    system.spawn_root_actor::<Distributor, _, _>("distributor", 0);

    thread::sleep(std::time::Duration::from_secs(10));
    system.shutdown();
}
