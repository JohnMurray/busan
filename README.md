# busan

[![Crates.io][crates-badge]][crates-url]
[![Build Status][actions-badge]][actions-url]
[![GNU GPL v3 licensed][gpl-badge]][gpl-url]

  [crates-badge]: https://img.shields.io/crates/v/busan.svg
  [crates-url]: https://crates.io/crates/busan
  [actions-badge]: https://github.com/JohnMurray/busan/actions/workflows/ci.yaml/badge.svg
  [actions-url]: https://github.com/JohnMurray/busan/actions/workflows/ci.yaml
  [gpl-badge]: https://img.shields.io/badge/license-GPL-blue.svg
  [gpl-url]: https://github.com/JohnMurray/busan/blob/main/LICENSE

\[[decision_log](https://github.com/JohnMurray/busan/tree/main/decisions)\]

Busan is an actor implementation for Rust that is currently under heavy development
and is experimental in nature.


----
## Raw Notes

### Task Scheduler
  + Thoughts:
    + actors are both state + workers/units of work
    + messages sent represent units of work that must be assigned to an actor
    + actors should be somewhat "sticky" to a thread
    + I/O model
        + forget about this for now
    + routing
      + addresses are composed of system + executor + actor
      + destination lookup:
        + Assume that all current lookups and sends are local
        + executor names could be shared by the control thread to all executors through
          the command protocol
        + actor names can be kept local (to avoid a lot of data redundancy as the system
          should handle _many_ more actors than executors)
        + executors can be the final router to assign messages to inboxes
      + execution flow
        + executors run a loop
          + check for updates on the command queue - process all messages
          + check for messages on the routing queue - process N messages
          + check for pending work on actors, run until:
            + command queue has messages
            + routing queue is at 80% capacity
            + all work is finished

### Threading Model
  + Each instance of the executor is independent and runs on their own thread
  + There is a global store of actors and messages
    + should the message and actors be tightly or loosely coupled?
  + Instances pull work from a common queue
  + __Question:__ Are actors sticky to an executor or are they moved around?


### Properties of Actors and Messages
  + All messages are protobuf and immutable
  + Messages must always be assumed to be serialized, but not guaranteed



### Thoughts on behaviors
  + all messages are protobuf
  + behaviors are grouping (maybe dynamic) of message processors
  + There is likely an API here to pattern match over the message types (via protobuf's reflection libs)
    to find the right processor. This could be hidden and a processor could be as simple as a lambda

```rust
let processor: Processor = processor(|msg: HelloActor| {
    // process the HelloActor message
});

fn processor<T: Message>(f: Fn(T) -> ()) -> Processor {
    // create a wrapper around F that checks the Message against the type T via
    // protobuf reflection
}
trait Processor {
    fn matches(&self, msg: Message) -> bool;
    fn process(&self, msg: Message);
}
```

  + Behaviors should easily be able to be grouped together

```rust
let group = vec![behavior1, behavior2, behavior3];
```

  + Actors define an initial behavior group as the receive function
  + At any point in time, the actor could call the `become(behavior_group)` function to change
    the behavior of the actor
  + The _actual_ `receive` method would be internal/private implementation detail of the Actor
    that uses the current behavior group to process messages

__open questions__
  + How does this set of behaviors interact with the actor's state?
