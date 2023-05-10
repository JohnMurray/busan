use crate::actor::{Actor, ActorAddress, ActorInit, Context};
use crate::message::{Message, ToMessage};
use std::cell::RefCell;
use std::iter::Cycle;
use std::slice::Iter;
use std::vec::IntoIter;

struct LoadBalancer {
    actors: Vec<ActorAddress>,
}

impl Actor for LoadBalancer {
    fn receive(&mut self, ctx: Context, msg: Box<dyn Message>) {
        todo!()
    }
}

impl ActorInit for LoadBalancer {
    type Init = ();
    fn init(init_msg: &Self::Init) -> Self
    where
        Self: Sized + Actor,
    {
        todo!()
    }
}
