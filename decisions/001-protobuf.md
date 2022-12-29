# Decision 001: Protobuf Messages
__2022-12-28__


TK - things to cover:
  + location transparency
  + protocol/message evolution
  + interoperability between separate systems


## Context/Problem

A core tenant of the Actor Model is communication through messages. This is separate from
say a thread based model in which memory is shared and coordinated through locks. While
sharing messages has positive side-effects that may be realized later in a project's
life cycle, such as location transparency, it is easy to violate this by sharing pointers
and indirectly causing shared memory.


## Options

List the options that are available. This section may also list some of the pros and cons (in an objective fashion)

## Choice

Detail which of the previous options were chosen. Now it's time to argue based on the pros and cons and the given
context why this was the best choice to be made.
