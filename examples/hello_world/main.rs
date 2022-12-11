extern crate busan;

use busan::system::ActorSystem;

fn main() {
    let system = ActorSystem::init();
    println!("Hello, world!");
}