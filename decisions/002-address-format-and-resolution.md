# Decision 001: Protobuf Messages
__2023-02-10__

## Context/Problem

The following are properties intrinsic to actor systems:

  + Actors communicate asynchronously by sending messages
  + Actor state is strongly encapsulated


## Constraints

At some point in the future, Busan must be able to support (or be extended to support) distributed actors
with the following properties:

  + Location transparency - Actors may be local or remote, but communication patterns are
    consistent regardless of location.

For these to be true 

Overview of the problem that we're trying to solve or some context/background information

## Options

List the options that are available. This section may also list some of the pros and cons (in an
objective fasion)

## Choice

Detail which of the previous options were chosen. Now it's time to argue based on the pros and cons
and the given context why this was the best choice to be made.