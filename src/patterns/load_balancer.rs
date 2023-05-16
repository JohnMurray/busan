use crate::actor::proto as actor_proto;
use crate::actor::{Actor, ActorAddress, ActorInit, Context};
use crate::message::common_types::impl_busan_message;
use crate::message::{Message, ToMessage};
use std::cell::RefCell;
use std::iter::Cycle;
use std::slice::Iter;
use std::vec::IntoIter;

// TODO: Create some initialization functions for the LB in the parent module
//       load_balancer(ctx, ROUND_ROBIN, actors);

// TODO: Things needed to fully implement a load balancer
//       + [ ] Test support for doing an end-to-end integration test (test system/executor)
//       + [ ] Shutdown component of lifecycle management

/// TODO: Write some docs
struct LoadBalancer {
    actors: Vec<ActorAddress>,
    strategy: Strategy,

    strategy_state: StrategyState,
}

impl Actor for LoadBalancer {
    fn receive(&mut self, ctx: Context, msg: Box<dyn Message>) {
        let actor = &self.actors[self.next()];
        ctx.send_message(actor, msg);
    }
}

impl LoadBalancer {
    fn new(actors: Vec<ActorAddress>, strategy: Strategy) -> Self {
        Self {
            actors,
            strategy,
            strategy_state: match strategy {
                Strategy::RoundRobin => StrategyState::RoundRobin { index: 0 },
                Strategy::Random => StrategyState::Random,
            },
        }
    }

    fn next(&mut self) -> usize {
        match &mut self.strategy_state {
            StrategyState::RoundRobin { index } => {
                *index = (*index + 1) % self.actors.len();
                *index
            }
            StrategyState::Random => rand::thread_rng() % self.actors.len(),
        }
    }
}

impl ActorInit for LoadBalancer {
    type Init = Init;

    fn init(init_msg: Self::Init) -> Self
    where
        Self: Sized + Actor,
    {
        /// This should be handled by utility code used to create load-balancers,
        /// so just some additional validations to ensure the addresses are present
        /// and the strategy is valid.
        debug_assert!(init_msg.nodes.is_some());
        debug_assert!(match init_msg.strategy {
            x if x == Strategy::RoundRobin as i32 => true,
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
                x if x == Strategy::RoundRobin as i32 => Strategy::RoundRobin,
                x if x == Strategy::Random as i32 => Strategy::Random,
                _ => Strategy::RoundRobin,
            },
        )
    }
}

include!(concat!(env!("OUT_DIR"), "/patterns.load_balancer.rs"));
impl_busan_message!(Init);

enum StrategyState {
    RoundRobin { index: usize },
    Random,
}
