# busan


## Task Scheduler
  + Thoughts:
    + actors are both state + workers/units of work
    + messages sent represent units of work that must be assigned to an actor
    + actors should be somewhat "sticky" to a thread
    + I/O model
        + forget about this for now

### Threading Model
  + Each instance of the executor is independent and runs on their own thread
  + There is a global store of actors and messages
    + should the message and actors be tightly or loosely coupled?
  + Instances pull work from a common queue
  + __Question:__ Are actors sticky to an executor or are they moved around?


## Properties of Actors and Messages
  + All messages are protobuf and immutable
  + Messages must always be assumed to be serialized, but not guaranteed


