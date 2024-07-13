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
- problem
    - function pointers are pretty tricky
        - can't take a function pointer to a method on a trait object
        - weak references are created with unsafe blocks, and I _really_ want
          to keep unsafe out of busan in its entirety

- I also want the "receive" function to be able to use the same set of matchers



Questions
---------
  - Is the `self` parameter an actual keyword?
    - it _is_ a keyword, but it has some flexibility on the type it can be. Update: no it doesn't. Not really.
    - https://doc.rust-lang.org/std/keyword.self.html
  - Can I use `self` as a parameter name?
    - upon investigation, nope. I can't use self unless it refers (direct or indirect) to the "expected" Self type

*/

/* This is an impl that I started with that's based on behavior "objects" but
had a flaw in that it can't pull in the local scope of the actor */
use crate::actor::Actor;
use std::any::Any;

pub struct Behavior<T: Actor> {
    matcher: fn(&dyn Any) -> bool,
    handler: fn(&mut T, &dyn Any) -> (),
}

impl<T: Actor> Behavior<T> {
    // TODO: Remove later, just was pretty annoyed for right now
    #[allow(dead_code)]
    fn apply(&self, msg: &dyn Any, actor: &mut T) -> bool {
        if (self.matcher)(msg) {
            (self.handler)(actor, msg);
            return true;
        }
        false
    }
}

// macro_rules! handle {
//     ($var:ident : $ty:ty, $b:block) => {
//         struct T{}
//         impl Behavior2<$ty> for T {
//             fn matcher(&self, msg: &dyn Any) -> bool {
//                 msg.is::<$ty>()
//             }
//             fn handler(self: &mut T, msg: &dyn Any) {
//                 let msg = msg.downcast_ref::<$ty>().unwrap();
//                 $b
//             }
//         }
//         let f = |$var: $ty| $b;
//         let x = 5;
//         println!("hello, world");
//         println!("second thing");
//     };
// }

pub fn test() {
    // handle!(self: String, {
    //     println!("Got a string: {}", self);
    // });
    use crate::actor::{ActorInit, Context};
    use crate::message::common_types::{I32Wrapper, StringWrapper};
    use crate::message::Message;

    trait Behavior2<T: Actor, M: Message> {
        fn matcher(&self, msg: &dyn Message) -> bool;
        fn handler(&mut self, msg: &dyn Message);
    }

    struct Ping {}
    impl ActorInit for Ping {
        type Init = I32Wrapper;
        fn init(_init_msg: Self::Init) -> Self
        where
            Self: Sized + Actor,
        {
            Ping {}
        }
    }
    impl Actor for Ping {
        fn receive(&mut self, ctx: Context, msg: Box<dyn Message>) {
            // Print the message and respond with a "ping"
            if let Some(str_msg) = msg.as_any().downcast_ref::<StringWrapper>() {
                println!("received message: {}", str_msg.value);
                ctx.send(ctx.sender(), "ping");
            }
        }
    }

    impl Behavior2<Ping, StringWrapper> for Ping {
        fn matcher(&self, msg: &dyn Message) -> bool {
            msg.as_any().downcast_ref::<String>().is_some()
        }
        fn handler(&mut self, msg: &dyn Message) {
            let msg = msg.as_any().downcast_ref::<String>().unwrap();
            println!("Got a string: {}", msg);
        }
    }
}
