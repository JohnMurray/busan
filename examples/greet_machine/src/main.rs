use busan::message::common_types::I32Wrapper;
use busan::prelude::*;

fn main() {
    let mut system = ActorSystem::init(ActorSystemConfig::default());
}

struct GreetMachine {
    num_greeters: i32,
}

impl ActorInit for GreetMachine {
    type Init = I32Wrapper;

    fn init(init_msg: Self::Init) -> Self
    where
        Self: Sized + Actor,
    {
        GreetMachine {
            num_greeters: init_msg.value,
        }
    }
}

impl Actor for GreetMachine {
    fn before_start(&mut self, mut ctx: Context) {
        for n in 0..self.num_greeters {
            ctx.spawn_child("greeter", n);
        }
    }

    fn receive(&mut self, ctx: Context, msg: Box<dyn Message>) {
        todo!()
    }
}
