# Decision 001: Protobuf Messages
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


## Options


## Choice

Detail which of the previous options were chosen. Now it's time to argue based on the pros and cons
and the given context why this was the best choice to be made.