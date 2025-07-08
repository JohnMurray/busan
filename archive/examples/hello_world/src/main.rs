use busan::actor::{Actor, ActorInit};
use busan::config::ActorSystemConfig;
use busan::message::common_types::StringWrapper;
use busan::message::Message;
use busan::system::ActorSystem;

fn main() {
    // Create a default actor system. This is the root of what will house and run our
    // actors, similar to a thread-pool for threads (but more sophisticated).
    let mut system = ActorSystem::init(ActorSystemConfig::default());

    // Create our initialization message and instruct the system to spawn the actor.
    // This will be the root actor and will be responsible for performing work and ultimately
    // shutting down the system.
    let init = proto::Init {
        greeting: "Hi there!".to_string(),
    };
    system.spawn_root_actor::<GreetActor, _, _>("greeter", init);

    // After spawning the root actor, we can block on the system to complete.
    system.await_shutdown();
}

// Import our proto messages for our actor. In this example, we're just
// importing the proto used for initializing the actor.
mod proto {
    include!(concat!(env!("OUT_DIR"), "/hello_world.rs"));
}

struct GreetActor {
    greeting: String,
}

// Define our initialization trait
impl ActorInit for GreetActor {
    type Init = proto::Init;

    fn init(init_msg: Self::Init) -> Self {
        println!("spawning greet actor");
        GreetActor {
            greeting: init_msg.greeting.clone(),
        }
    }
}

impl Actor for GreetActor {
    // Define our initial startup logic here in the before_start. This is invoked by the
    // system once the actor has been assigned to an executor and thus can spawn child
    // actors or send messages.
    fn before_start(&mut self, ctx: busan::actor::Context) {
        // Send a greeting message to ourselves
        ctx.send(ctx.address(), &self.greeting);
    }

    fn receive(&mut self, mut ctx: busan::actor::Context, msg: Box<dyn Message>) {
        // Match the incoming message against a set of types that we expect. When we
        // receive a string, print out the greeting.
        match msg.as_any().downcast_ref::<StringWrapper>() {
            Some(msg) => {
                println!("received message: {}", msg.value);
                ctx.shutdown();
            }
            None => self.unhandled(ctx, msg),
        }
    }
}
