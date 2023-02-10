# Decision 001: Protobuf Messages
__2023-02-10__

## Context/Problem

The following are properties intrinsic to actor systems:

  + Actors communicate asynchronously by sending messages
  + Actor state is strongly encapsulated
  + Actors (with the exception of the "root" actor) are created by other actors


## Constraints

My goals with Busan at the time of writing is to build a robust actor implementation, this means
there are some general constraints for any system we build, but the ones that were are relevant to
this problem are:

  + __Location transparency__ - Actors may be local or remote, but communication patterns
    are consistent regardless of location. This is necessary pre-condition to building
    great distributed actor support.
  + __Supervision Trees__ - Actors naturally create tree-like structures given that actors
    may only be spawned by other actors, creating a parent/children relationship. While this
    relationship does not _have_ to be encoded in address format, the address should not
    _prevent_ the encoding of this information somewhere (even if private).
  + __Optimization Friendly__ - Addressing (and address resolution) should not be too perscriptive
    in how the underlying system works. The solution does not need to be performant today, but
    it shouldn't make internal changes later more difficult.
      + Example: Using the ID of the executor in the address would prevent us from moving actors
        between schedulers.
  + __User Friendly / Meaningful__ - The address is a very user-facing aspect of the framework.
    As such, it should be friendly to users and ideally the value of an address should be
    meaningful. For example, if the address is simply a hash (e.g. SAH256), that is not a
    meaningful value for users if they were to print out the address or inspect it during
    debugging.

For these to be true 

Overview of the problem that we're trying to solve or some context/background information

## Options

List the options that are available. This section may also list some of the pros and cons (in an
objective fasion)

## Choice

Detail which of the previous options were chosen. Now it's time to argue based on the pros and cons
and the given context why this was the best choice to be made.