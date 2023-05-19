use crate::actor::{Actor, ActorAddress, ActorInit, Context};
use crate::message::Message;

// TODO: Create some initialization functions for the LB in the parent module
//       load_balancer(ctx, ROUND_ROBIN, actors);

// TODO: Things needed to fully implement a load balancer
//       + [ ] Test support for doing an end-to-end integration test (test system/executor)
//       + [ ] Shutdown component of lifecycle management
//           + [ ] Including a death-watch mechanism so that nodes can be removed from LBs

// TODO: Write some docs
struct LoadBalancer {
    actors: Vec<ActorAddress>,

    strategy_state: StrategyState,
}

impl Actor for LoadBalancer {
    fn receive(&mut self, ctx: Context, msg: Box<dyn Message>) {
        let next = self.next();
        let actor = &self.actors[next];
        ctx.send_message(actor, msg);
    }
}

impl LoadBalancer {
    fn new(actors: Vec<ActorAddress>, strategy_state: StrategyState) -> Self {
        Self {
            actors,
            strategy_state,
        }
    }

    fn next(&mut self) -> usize {
        match &mut self.strategy_state {
            StrategyState::RoundRobin { index } => {
                *index = (*index + 1) % self.actors.len();
                *index
            }
            StrategyState::Random => 0, // rand::thread_rng() % self.actors.len(),
        }
    }
}

impl ActorInit for LoadBalancer {
    type Init = proto::Init;

    fn init(init_msg: Self::Init) -> Self
    where
        Self: Sized + Actor,
    {
        // This should be handled by utility code used to create load-balancers,
        // so just some additional validations to ensure the addresses are present
        // and the strategy is valid.
        debug_assert!(init_msg.nodes.is_some());
        debug_assert!(match init_msg.strategy {
            x if x == proto::Strategy::RoundRobin as i32 => true,
            x if x == proto::Strategy::Random as i32 => true,
            _ => false,
        });

        Self::new(
            init_msg
                .nodes
                .map(|n| n.addresses)
                .unwrap_or_else(Vec::new)
                .into_iter()
                .map(|a| a.into())
                .collect(),
            match init_msg.strategy {
                x if x == proto::Strategy::RoundRobin as i32 => {
                    StrategyState::RoundRobin { index: 0 }
                }
                x if x == proto::Strategy::Random as i32 => StrategyState::Random,
                _ => StrategyState::Random,
            },
        )
    }
}

pub(crate) mod proto {
    use crate::message::common_types::impl_busan_message;
    use crate::message::Message;
    include!(concat!(env!("OUT_DIR"), "/patterns.load_balancer.proto.rs"));
    impl_busan_message!(Init);
}

enum StrategyState {
    RoundRobin { index: usize },
    Random,
}
