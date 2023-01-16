# Decision Logs

This folder contains a set of (roughly ordered) decision logs regarding Busan. All major technical
decisions should (if I wasn't feeling super lazy that day) have an accompanying decision log. But
these take time to write and aren't super fun, so I've only included these for things where picking
choice A means that choice B is not really an option anymore.

For example, forcing all messages to be Protobuf could mean __not__ being able to share things like
pointers or locks (because those things can't be serialized into Protobuf). This warrants a decision
log. Alternatively, choosing to implement a thread-based executor, because the interface is generic,
would _not_ prevent the creation of say a non-blocking IO executor. In this case, because the
alternative is still possible, it's not really worth a log entry for.

  + [000 Busan - Why write a new framework?](https://github.com/JohnMurray/busan/blob/main/decisions/000-busan.md)
  + [001 Protobuf Messaes](https://github.com/JohnMurray/busan/blob/main/decisions/001-protobuf.md)


## Template (for my own copy/pasta)

```markdown

# Decision XXX: <TITLE>
__<DATE>__

## Context/Problem

Overview of the problem that we're trying to solve or some context/background information

## Options

List the options that are available. This section may also list some of the pros and cons (in an
objective fasion)

## Choice

Detail which of the previous options were chosen. Now it's time to argue based on the pros and cons
and the given context why this was the best choice to be made.

```
