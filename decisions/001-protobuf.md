# Decision 001: Protobuf Messages
__2022-12-28__


## Context

A core tenant of the Actor Model is communication through messages. This is separate from
say a thread-based model in which coordination and communication is _typically_ handled through
shared memory and involving locks and/or atomic operations. The Actor Model simplifies the
communication and coordination model by limiting the mechanisms to exclusively message passing.

This mechanism is both enforced by and allows for some notable properties of an Actor Model
based system:

  + Actors are strongly encapsulated
  + Actors are single-threaded
  + Simplification of logic within an actor - no use of synchronization primitives
  + Actors are easily distributed across multiple processes/machines with little to
    no change in program logic.

Strong encapsulation is worth defining here and it essentially means that each actor has
some local (mutable) state that can _only_ be accessed by the actor. This state is mutated
based on messages received and could by shared by sending messages. No element outside of
the actor may access (read or write) the internal state of an actor.

With this definition, we can see how the above points are all interconnected:
  + Actors are units of concurrency and inherently "single threaded"
  + Single threaded, strongly encapsulated actors have no use for synchronization primitives
  + Strong encapsulation with message passing means it doesn't matter if the
    actor being communicated with is on the same thread, process, machine, or even
    datacenter.

There are other properties that are often ascribed to actor model based systems as well such
as fault tolerance or self-healing. While this is not strictly related to message passing,
recovering from failure in a transparent manner is dependent on an actor's strong
encapsulation.  Recovery in actor systems may result in the re-creation or "resetting" of
internal state, but for that to be easy (or at least feasible), an actor must be strongly
encapsulated such that local mutation of internal state does not have side-effects outside
of the current actor.

This is very likely missing some important use-cases, but it is sufficient to say that
strongly encapsulated actors, enabled by a message-passing communication mechanism is
essential to a healthy actor implementation.

## Problem

This is not a "problem" in the traditional sense. What I've described so far is really a
set of requirements for how Busan will be built. Where there is possibly friction and where
a decision must be made is in the construction of an API.

All of the context above describes what is desired in a theoretical system, but how does
that translate to constructing an API for sending messages?

The minimum requirement for sending a message across threads (and of course Busan will build
on top of existing concurrency primitives) is to implement `Send` (and maybe `Sync`). In our
case we would absolutely _not_ want `Sync`, but [negative_impls][neg_impl] is not a standard
feature. And even if it were, a struct could contain a field that was `Sync + Send` (such as
an `Arc`). Allowing `Sync` to slip in could violate our strong encapsulation.

Additionally, location transparency is an initial goal (even if distributed message sending
is still a ways off). This means beyond `Send` to allow for sending between threads, we need
some form of serialization to support sending between processes or machines.


  [neg_impl]: https://github.com/rust-lang/rust/issues/68318

## Options

### User Choice

An option is to _not_ put any requirements on message sending, other than what the Rust
type system will enforce. This would provide users the opportunity to side-step the
strong isolation of the Actor Model and would also require a separate API for messages
traveling outside of the process.

#### Pros

  + Freedom and flexibility in types that can be sent, without need of specific serialization
    formats or use/implementation of special macros/traits on types.
  + Choice is serialization format (for remote messages)

#### Cons

  + Easy to violate principals of the Actor Model (potential foot-guns)
  + No location transparency
  + Separate APIs for local vs remote message passing
  + Potential blocker for future functionality or utilities, or limits their use to a sub-set
    of "good" applications
  + Increased user complexity along with explosion of choices in how to construct and
    coordinate actors.
  + Strong use of "best practices" would be required to reap full benefits of the Actor Model


### Allow/Deny-List Specific Types

`Sync` is the bit that seems to really get types in trouble when it comes to adhering
to the strong isolation principals of the Actor Model. An idea could be to simply block
implementations of this trait (e.g. `Arc` or `Rc`).

I don't have strong evidence here and this is mostly intuition. Type systems are complex,
powerful ones even more so. I suspect, for the sophisticated user, this would be easy to
circumvent. It may _additionally_ be the case that this is too complex of a requirement to
implement soundly.

If the feature is too complex to get right or easily circumvented, then it would be more
beneficial (for both library maintainers and users) if this choice is deferred to the user,
as limited value is being provided.


### Mandatory Serialization Format

All message that are sent must be an immutable, serializable message implemented using a common
serialization format. This could be something Rust-specific like `serde` or it could be a more
powerful tool such as Protobuf or Thrift. All message sends, local or remote, would be serialized
at the sender and deserialized on the receiver.


#### Pros

  + Messages types can still implement `Sync` while being guaranteed memory is not shared.
  + Strict adherence to the Actor Model principles, so all promises are satisfiable
    (location transparency, actor isolation, etc).
  + "Best Practice" is the default operating mode and does not require users to deeply understand
    the Actor Model and its trade-offs.
  + Future features will be applicable to ~all applications

#### Cons

  + Overhead of serializing/deserializing on local message sends
  + Requires using a serialization library for _every_ message. This may lead to a slightly
    awkward API for users.


#### Implementation Notes

__Cost:__ There is an obvious (and slightly ridiculous) cost of serialization when it is applied
to _every_ message. Say for example a message is being sent to a child actor running in the same
process and even possibly running on the same thread. Serializing this adds a rather unacceptable
amount of overhead for a "real" system. However, this serialization is our insurance that we're
following best practices and best practices should be the default operating mode.

There is an easy solution here which is to simply only turn on this behavior with debug builds.
This can be done with some macros, but imagine a send function that roughly looks like:

```rust
fn send(&self, mut msg: Message, to: ActorAddress) {
  // Macro that serializes and deserializes `msg` and re-assigns the value.
  // Is a no-op when in release mode (msg = msg)
  msg = debug_serialize!(msg);
  if to.is_local() {
    // Will not be serialized
    self.send_local(msg);
  }
  else {
    // Will be serialized
    self.send_remote(msg);
  }
}
```

This can help verify the correctness of the program while providing an optimization at runtime
and allowing for the library to be written with sane assumptions.


## Choice

__Mandatory Serialization Format__

The compromises involved in the other options hinder the potential utility of the library as an
accurate implementation of the Actor Model and leave too many options on the table. At a certain
point, too many options begs the question why there is a library or framework at all.

I am of a strong opinion that a library such as this should provide best practices as a default
operating mode. Users should be able to write good, correct code first and later come to understand
the principles of the Actor Model and how they're at play. Actor-based programming is different
enough without also having to understand the more intricate parts of the theory as a prerequisite
to being productive.

It's worth noting that something like a record class of _truly_ immutable data would have been a
good choice here, but I couldn't find anything that fit that mold in Rust. But I'm not expert and
it's possible that I'm missing a more idiomatic choice.

__Let's talk Protobuf__

This could maybe be another decision log, but I think the argument is pretty concise.

The decision is to use a serialization library, so it makes sense to standardize on one. Protobuf
makes the most sense here for the following reasons:

  + Protobuf is a well-defined specification that is neither language nor library dependent
    + Caveat: Busan's specific usage _may_ be library-specific, but more mentioning the
              serialization aspect.
  + Protobuf the most popular, structured serialization format; is cross-language; and used with
    gRPC.
    + This would make gRPC bridging very easy in the future (a gRPC entry-point to an actor system)
    + Remote actors would not _necessarily_ have to be written in Busan. Protobuf provides room
      for a standard to emerge.
  + Message evolution. Protobuf message evolution is a well understood problem and Protobuf provides
    mechanisms for forward and backwards compatibility. When developing a large-scale system, it
    makes sense to allow some amount of message version skew.
  + Protobuf is declared with a schema file and easily shared/re-used/referenced.

There really aren't many options for a serialization format that are as popular, well-known, and
well-supported as Protobuf. There may be "better" options out there, but in this case I think it
is worth it to go with a well-known/supported option.