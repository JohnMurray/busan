//! Demonstrate ACK messages by creating a "load balancer"" that sends work as soon
//! as the previous message has been acknowledged. Since message ACK's are based on
//! receipt in the actor's queue and not a signal for message processing, this isn't
//! a real load balancer.
use busan::actor::{Actor, ActorAddress, ActorInit, Context};
use busan::config::{ActorSystemConfig, ExecutorConfig};
use busan::message::common_types::{I32Wrapper, U32Wrapper};
use busan::message::system::Ack;
use busan::message::Message;
use busan::system::ActorSystem;
use log::info;
use std::thread;

struct Distributor {
    worker_count: u32,
    workers: Vec<ActorAddress>,
    work_ack_nonce: Vec<u32>,
    work_queue: Vec<u32>,
}
struct Worker {
    work_received: u32,
}

impl ActorInit for Distributor {
    type Init = U32Wrapper;

    fn init(init_msg: Self::Init) -> Self
    where
        Self: Sized + Actor,
    {
        println!("init distributor");
        Distributor {
            worker_count: init_msg.value,
            workers: vec![],
            work_ack_nonce: vec![],
            work_queue: (0..100).collect(),
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
        Worker { work_received: 0 }
    }
}

impl Actor for Distributor {
    fn before_start(&mut self, mut ctx: Context) {
        // Spawn all of our initial workers and begin assigning work as
        // soon as the worker is ready.
        self.workers = (0..=(self.worker_count))
            .map(|_| {
                let worker = ctx.spawn_child::<Worker, _, _>("worker", 0).await_unwrap();
                self.send_work(&mut ctx, &worker);
                worker
            })
            .collect();
        debug_assert!(!self.workers.is_empty());
    }

    fn receive(&mut self, mut ctx: Context, msg: Box<dyn Message>) {
        // If we receive an ack from a worker, send the next work item to that
        // actor.
        if let Some(ack) = msg.as_any().downcast_ref::<Ack>() {
            info!("Received ack({}) from {}", ack.nonce, ctx.sender());
            if self.work_ack_nonce.contains(&ack.nonce) {
                self.work_ack_nonce = self
                    .work_ack_nonce
                    .iter()
                    .map(|n| *n)
                    .filter(|n| *n != ack.nonce)
                    .collect();
                let sender = ctx.sender().clone();
                self.send_work(&mut ctx, &sender);
            }
            if self.work_ack_nonce.is_empty() && self.work_queue.is_empty() {
                info!("All work has been compelted. Shutting down.");
                ctx.shutdown();
            }
        }
    }
}

impl Distributor {
    fn send_work(&mut self, ctx: &mut Context, worker: &ActorAddress) {
        if let Some(work) = self.work_queue.pop() {
            info!("pop'ing work({}) from queue", work);
            self.work_ack_nonce.push(ctx.send_with_ack(worker, work));
        }
    }
}

impl Actor for Worker {
    fn receive(&mut self, ctx: Context, msg: Box<dyn Message>) {
        if let Some(work_msg) = msg.as_any().downcast_ref::<U32Wrapper>() {
            info!("received work({}) from {}", work_msg.value, ctx.sender());
            thread::sleep(std::time::Duration::from_millis(50));
            self.work_received += 1;
        }
    }

    fn before_stop(&mut self, _: Context) {
        info!("total work-items received: {}", self.work_received);
    }
}

fn main() {
    env_logger::Builder::new()
        .filter_level(::log::LevelFilter::Debug)
        .init();

    let mut system = ActorSystem::init(ActorSystemConfig {
        executor_config: ExecutorConfig {
            num_executors: 10,
            ..ExecutorConfig::default()
        },
    });
    system.spawn_root_actor::<Distributor, _, _>("distributor", 10u32);
    system.await_shutdown();
}
