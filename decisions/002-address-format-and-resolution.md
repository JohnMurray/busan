# Decision 003: Address Format & Resolution
__2023-02-10__

## Context/Problem

Message based communication between strongly encapsulated actors is the foundation of the
actor model. Actors are also a unit of concurrency and thus the combination of these attributes
leads toward the direction of needing "references" or "addresses" by which to direct
communication.

## Constraints

+ __Location transparency__ - Actors may be local or remote, but communication patterns are
  consistent regardless of location. This is necessary pre-condition to building great distributed
  actor support.
+ __Supervision Trees__ - Actors naturally create tree-like structures given that actors may only
  be spawned by other actors, creating a parent/children relationship. While this relationship does
  not _have_ to be encoded in address format, the address should not _prevent_ the encoding of this
  information somewhere (even if private).
+ __Optimization Friendly__ - Addressing (and address resolution) should not be too perscriptive
  in how the underlying system works. The solution does not need to be performant today, but it
  shouldn't make internal changes later more difficult.
    + Example: Using the ID of the executor in the address would prevent us from moving actors
      between schedulers.
    + Example: Addresses are typically _highly shared_ objects within an actor system, these should
      not be insanely expensive objects.
+ __User Friendly / Meaningful__ - The address is a very user-facing aspect of the framework.  As
  such, it should be friendly to users and ideally the value of an address should be meaningful.
  For example, if the address is simply a hash (e.g. SAH256), that is not a meaningful value for
  users if they were to print out the address or inspect it during debugging.
+ __Uniqueness__ - Names must be unique for the lifetime of the actor system. This means if an actor
  is spanwed with a given name of `"A"` and that actor terminates. Another actor with the given name
  `"A"` must not have the same address. This prevents accidental communication lines and encourages
  that addresses are discovered through direct address sharing.


## Format Options

### Flat Naming

+ The actor system is encoded with `local:/` prefix to signify an address local to the current
  actor system (with the assumption that remote systems will have a different prefix)
+ Actors are created with a name given by the user
+ Uniqueness is guaranteed by appending a number to the end of the given name (e.g. `-0`, `-1`, etc.)

The API from the user would look like:

```rust
impl Actor for Ping {
    fn before_start(&mut self, ctx: Context) {
        let addr = ctx.spawn_child::<_, Pong>("pong".to_string(), &());
        let addr2 = ctx.spawn_child::<_, Pong>("pong".to_string(), &());
    }
}
```

If we were to debug print the value of `addr` we would see:

```text
local:/pong
```

and if we were to debug print the value of `addr2` we would see:

```text
local:/pong-1
```

`local:/pong` is synonymous with `local:/pong-0`, but the `-0` is excluded for general readability
by users.

Parents are encoded with an internal pointer. In the case of our example, the parent is the `Ping`
actor. An example of what this could look like:

```rust
println!("{:?} -> {:?}", addr.parent, addr); 
// local:/ping-3 -> local:/pong
```

Finding the hierarchy of parents could be done by repeatedly calling `.parent` until either the root
Actor is required or until `.parent` returned some form of "empty" value.


### Hierarchic Naming

Building on __Flat Naming__, the hierarchy could be encoded in the address directly using a path
structure (similar to navigating a file-system or traditional web-server). Construction would look
the same, except the internal representation would be different:

```rust
impl Actor for Ping {
    fn before_start(&mut self, ctx: Context) {
        let addr = ctx.spawn_child::<_, Pong>("pong".to_string(), &());
        let addr2 = ctx.spawn_child::<_, Pong>("pong".to_string(), &());

        debug!("{:?}", addr);
        // local:/root/ping-2/pong

        debug!("{:?}", addr2);
        // local:/root/ping-2/pong-1
    }
}
```

The parent is now encoded in the path, so a separate reference does not need to be tracked
in the address object. Instead, the address to the parent can be computed on the fly. Something
like the following would be possible:

```rust
def parent(&self) -> Address {
    let mut path = self.path.clone();
    path.pop();
    Address::new(path)
}
```

### Hashed Naming

Using some form of hashing to create unique actor names doesn't satisfy the constraints that
addresses are user-friendly and meaningful and doesn't have any other advantages over flat
naming.

### Executor Based Pathing

Make use of either flat or hierarchical naming, but include path information that identifies
the actor's physical location within the actor system (such as an "executor ID").

This violates the constraint of not preventing future optimizations (e.g. actor re-balancing).
However, this does have friendlier resolution mechanics as is discussed in the resolution options.

## Resolution Options

An actor address can be thought of in two parts, there is the representation of the address, such
as the URI `local:/root/ping-2/pong-1` and then there are the mechanisms of how messages are
transmitted to the actor. In the current implementation (at time of writing), this is represented
by a `crossbeam_channel` `Sender` and `Receiver` pair.  __Resolution__ then can be defined as
converting the representation of the address into the `crossbeam_channel::Sender`.

### Global Registry Lookup

This is the most straight-forward approach to address resolution and can be separated from the
chosen address format/representation. A global registry is maintained by the actor system in a
centralized component (such as the management thread) and requests can be made (synchronous or
asynchronous) to the registry to resolve an address.

The only requirement for this approach is that addresses must be registered upon actor creation.
Actors may move between executors and will not require any updates to the registry.

If actor creation is a centralized operation (in which the management thread is responsible for
scheduling the actor to an executor), then the registry can be updated at the same time. Actor
creation is _currently_ a centralized operation at time of writing.

The main drawback of this approach is the cost of coordinating across multiple executors (which
likely means threads). This limitation could be mitigated through a few different approaches:
  + Having a secondary "cache" registry local to the executorso
  + Optimistically providing resolution in select circumstances where the data may already
    be available (e.g. Parent -> Child, Child -> Parent)

### Path Based Identification

Take advantage of the fact that actors are created by other actors. Thus, the parent should have
an easy time getting access to the fully resolved address of the child. Even if the implementation
uses a centralized creation mechanism, it should be easy (and generally expected/advantageous) for
the parent to have a resolved address to the child.

In a path-based representation, it's possible to traverse the hierarchy of actors, starting at the
root actor.

This has some advantages such as not requiring a centralized registry, but also does not make any
assumptions about how an actor is scheduled (initially or later). However, some significant
drawbacks exist:
  + Resolution becomes an inherently asynchronous operation (as all actor communication is).
  + The cost of performing a resolution is relative to an actors position in the tree.

### Executor Based Identification

If the address format includes identifying information about the executor, then the resolution
can skip the global registry and perform a lookup directly with the executor. This combines the
lookup from the global registry approach with (some of )the decentralized nature of the
path-based approach.

The registry is fragmented across the set of available executors and thus removes some of the
coordination cost of a global registry. However, the executor registry requires updating both
on initial actor creation, as well as when an actor is moved between executors. Since the executor
is encoded into the address representation, this would also require _forwarding_ resolution
to another executor when a move has occurred.

## Choice

__Hierarchic Naming with Global Registry Lookup__.

Hierarchic naming best suits the constraints while providing the best UX. The global registry
provides an extremely simple mechanism for resolving addresses with clear paths for future
optimizations (if necessary).