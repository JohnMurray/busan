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


## Options

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

Finding the heirarchy of parents could be done by repeatedly calling `.parent` until either the root
Actor is required or until `.parent` returned some form of "empty" value.


### Hierarchic Naming

## Choice

Detail which of the previous options were chosen. Now it's time to argue based on the pros and cons
and the given context why this was the best choice to be made.