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

Additionally, location transparency is an initial goal. This means beyond `Send` to allow
for sending between threads, we need some form of serialization to support sending between
processes or machines.


  [neg_impl]: https://github.com/rust-lang/rust/issues/68318

## Options

### Defer to the User

The simplest option is to simply "do nothing". In order to send messages locally (within the
same process), it is only required that types implement the `Send` trait (and maybe the `Sync`
trait). If a message needs to be sent outside of the current process, then it is up to the user
to either ensure the data can be serialized or to explicitly handle serialization and
deserialization.

This is the most flexible option, but has a few drawbacks:
  + This is a violation of the Actor model, which means several promises (actor isolation,
    simpler concurrency, location transparency, etc) are no longer feasible within the
    framework.
  + This exposes a potentially controversial amount of choice to the user.
  + This handicaps future development of features/utilities or adds complexity of only
    being able to apply features/utilities to a subset of applications.
  + This requires additional rigor from developers to adhere to best practices.


### Allow/Deny-List Specific Types

Perhaps it is possible to block the types that are likely to cause "violations" of the
actor model, such as `Arc` or `RC`.

I don't have strong evidence here and this is mostly intuition. Type systems are complex,
powerful ones even more so. I suspect, for the sophisticated user, this would be easy to
circumvent. It may _additionally_ be the case that this is too complex of a requirement to
implement soundly.

If the feature is too complex to get right or easily circumvented, then it would be more
beneficial (for both library maintainers and users) if this choice is deferred to the user,
as limited value is being provided.


### Mandatory Serialization Format

Require that all messages passed are compatible with a standard serialization format, such
as Protobuf. Invoke serialization/deserialization with every message send to _fully ensure_
no state is shared, purposefully or inadvertently.

This approach is _ideal_ in that it most closely matches the ideal of the Actor Model with
respect to message passing. Messages are immutable bits of data which means that all of the
promises of the model are satisfiable, such as location transparency, actor isolation (as
sofar as inadvertent state sharing), TK ... (continue list of typical model advantages)

TK: Go on to talk about
  + protocol/message evolution (distributed systems)
  + interoperability between separate systems

TK - Cons
  + Cost
  + Awkward API
  + CPU Cost

### Parallel Serialization API

TK: This could be an _optional_ API, as in it could be combined with a non-serizlized option which lands it
    in the _realm_ of defer'ing to the user, but with added capabilities.

### TK: ???

TK: What other options are available? Need a bit of time to think on this


## Choice

TK: Serialization Library

TK: Having worked in dev-prod for a large organization, folks often just want to be told the right thing
    to do and get on with building cool stuff. Avoid paralysis of choice.

TK: The "right" thing needs to be super easy
TK: Achieve something that _looks_ like a hybrid model while still enforcing the "right" way. Use clever API
    tricks to make things nice, if need be.

