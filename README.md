# busan


## Task Scheduler
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


## Properties of Actors and Messages
  + All messages are protobuf and immutable
  + Messages must always be assumed to be serialized, but not guaranteed


