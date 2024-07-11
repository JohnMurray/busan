/*

match_message!(msg) {
    m: MyTypte => {

    },
}

thoughts:
- I want the behaviors to seem like they're apart of your actor, not like they
  are part of some other object. That means having access to the local state.
- What if:
    - the macro just defines a function that takes the actor as an input to
      self.
    - then the actor (or context) can have a "real_receive" that defaults to
      the current receive, but can be overriden by calling something like
      ctx.become(behavior)

- I also want the "receive" function to be able to use the same set of matchers
*/

/* This is an impl that I started with that's based on behavior "objects" but
had a flaw in that it can't pull in the local scope of the actor */
// use crate::actor::Actor;
// use std::any::Any;
//
// pub struct Behavior<T: Actor> {
//     matcher: fn(&dyn Any) -> bool,
//     handler: fn(&dyn Any, &mut T) -> (),
// }
//
// impl<T: Actor> Behavior<T> {
//     // TODO: Remove later, just was pretty annoyed for right now
//     #[allow(dead_code)]
//     fn apply(&self, msg: &dyn Any, actor: &mut T) -> bool {
//         if (self.matcher)(msg) {
//             (self.handler)(msg, actor);
//             return true;
//         }
//         false
//     }
// }
