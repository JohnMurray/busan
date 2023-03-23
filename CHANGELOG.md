## 0.1.2 through 0.2.0 [wip]

  + Added support for spawning actors (`0.1.2`)
  + Introduced runtime manager and round-robin spawn strategy (`0.1.3`)
  + Added configuration support for creating new ActorSystem
  + Implemented `ActorCell`
  + Implemented `ActorAddress` (for local routing only) and address resolution
  + Basic message sending
  + Pattern matching on received types (by use of `Any`) added. Uses newly
    added `Message` type (which subsumes `prost::Message`) to implement `as_any()`.

## 0.1.1

  + Added a CHANGELOG.md file :-)
  + New decision log entry: [000 - Busan][dl_000]
  + `thread_executor` moved into `executor` sub-module
  + Added `shutdown` and `await_shutdown` to `ActorSystem`

  [dl_000]: http://github.com/JohnMurray/busan/blob/master/decisions/000-busan.md
