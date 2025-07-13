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
use crate::actor::{Actor, Context};
use crate::message::Message;

pub struct BehaviorSet<A: Actor> {
    behaviors: Vec<Box<dyn Behavior<A>>>,
}

impl<A: Actor> BehaviorSet<A> {
    pub fn new(behaviors: Vec<Box<dyn Behavior<A>>>) -> Self {
        Self { behaviors }
    }

    pub fn can_handle(&self, msg: &dyn Message) -> bool {
        for behavior in self.behaviors.iter() {
            if behavior.is_match(msg) {
                return true;
            }
        }
        false
    }

    pub fn handle(&self, actor: &mut A, ctx: Context, msg: &dyn Message) -> Result<(), ()> {
        for behavior in self.behaviors.iter() {
            if behavior.is_match(msg) {
                behavior.handle(actor, ctx, msg);
                return Ok(());
            }
        }
        Err(())
    }

    pub fn is_empty(&self) -> bool {
        self.behaviors.is_empty()
    }

    pub fn empty() -> Self {
        Self {
            behaviors: Vec::new(),
        }
    }
}

trait Behavior<A: Actor> {
    fn is_match(&self, msg: &dyn Message) -> bool;
    fn handle(&self, actor: &mut A, ctx: Context, msg: &dyn Message);
}

// pub struct Behavior {
//     matcher: fn(&dyn Message) -> bool,
//     handler: fn(&mut dyn Actor, ctx: Context, &dyn Message) -> (),
// }

// impl Behavior {
//     pub fn new(
//         matcher: fn(&dyn Message) -> bool,
//         handler: fn(&mut dyn Actor, Context, &dyn Message) -> (),
//     ) -> Self {
//         Self { matcher, handler }
//     }
// }

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

#[cfg(test)]
mod test {
    use super::*;
    use crate::actor::{Actor, ActorInit, Context};
    use crate::message::common_types::{I32Wrapper, StringWrapper};
    use crate::message::Message;

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
        fn receive(&mut self, _: Context, _: Box<dyn Message>) {}

        fn init_state(&self) -> BehaviorSet {
            BehaviorSet::new(vec![Behavior {
                matcher: |msg| msg.as_any().downcast_ref::<StringWrapper>().is_some(),
                handler: |_actor, _, msg| {
                    let msg = msg.as_any().downcast_ref::<StringWrapper>().unwrap();
                    let a = _actor.as_any().downcast_ref::<Ping>().unwrap();
                    println!("Got a string: {}", msg.value);
                },
            }])
        }
    }

    #[test]
    pub fn test() {
        // impl Behavior2<Ping, StringWrapper> for Ping {
        //     fn matcher(&self, msg: &dyn Message) -> bool {
        //         msg.as_any().downcast_ref::<String>().is_some()
        //     }
        //     fn handler(&mut self, msg: &dyn Message) {
        //         let msg = msg.as_any().downcast_ref::<String>().unwrap();
        //         println!("Got a string: {}", msg);
        //     }
        // }
    }
}
/*
  Thought Experiment
    - The concerns around `self` are irrelevant (for now)
    - Behaviors are a clean way to implement the actor state machine and we can do this if
      we don't think about `self`.
    - What would an ideal API look like here?

What we have today:
----
    fn receive(&mut self, ctx: Context, msg: Box<dyn Message>) {
        // Print the message and respond with a "ping"
        if let Some(str_msg) = msg.as_any().downcast_ref::<StringWrapper>() {
            println!("received message: {}", str_msg.value);
            ctx.send(ctx.sender(), "ping");
        }
    }

What is not ideal about this?
    - The APIs are not composable in any way
    - Exposing the type-matching logic is gross _and_ an implementation detail that's
      leaking into userspace code
    -

Proposal #1
----
    fn receive(&mut self, ctx: Context, msg: Message) {
        match_message!(msg) {
            m: StringWrapper => {
                println!("received message: {}", m.value);
                ctx.send(ctx.sender(), "ping");
            },
            _ => {
                println!("received message: {:?}", msg);
            }
        }
    }

    GOOD
    - This is good for the simple case where you're not really using the actor as a
      state machine at all.
    - This matches more idiomatic rust code and pattern matching

    BAD
    - This isn't composible, there are no behavior objects being created/returned

Proposal #2
----
    fn starting_state(&mut self, _: Context) -> BehaviorSet {
        BehaviorSet::new(vec![
            Behavior::new(|msg| msg.is::<StringWrapper>(), |actor, ctx, msg| {
                let msg = msg.downcast_ref::<StringWrapper>().unwrap();
                ctx.send(ctx.sender(), "ping");
                actor.become(actor.next_state())   // STATE TRANSITION
            }),
            Behavior::new(|msg| true, |actor, msg| {
                println!("received message: {:?}", msg);
            }),
        ])
    }

    fn next_state(&self) -> BehaviorSet {
        BehaviorSet::new(vec![
            Behavior::new(|msg| msg.is::<StringWrapper>(), |actor, msg| {
                let msg = msg.downcast_ref::<StringWrapper>().unwrap();
                ctx.send(ctx.sender(), "ping-ping");
            }),
        ])
    }

    // This would become a default impl on Actor
    fn receive(&mut self, ctx: Context, msg: Box<dyn Message>) {
        self.behavior.receive(ctx, msg);
    }


    NOTES
    - Behaviors would need to live in the ActorState

    GOOD
    - These are composable, don't require any macros to get working

    BAD
    - Boilerplate is higher here, but could be improved with macros
    - Ergonomics of having to pass everything around (actor, context, etc) is worse
      than the current state and/or having access to `self`
    - Captures on the lambdas might be confusing because the storage for these behaviors
      is going to be outside the actor


Proposal #3
----
An addition onto Proposal #2 that introduces a few macros to make the
construction and usage of behaviors easier:

    fn starting_state(&mut self, _: Context) -> BehaviorSet {
        behaviors![
            match!<StringWrapper> => |actor, ctx, stringMsg| {
                println!("received message: {}", stringMsg); // stringMsg is a String
                ctx.send(ctx.sender(), "ping");
                actor.become(actor.next_state())  // STATE TRANSITION
            },
            default_match! => |actor, ctx, msg| {
                println!("received message: {:?}", msg);
            },
        ]
    }



*/
