## WIP (unreleased)

  + Added debug serialization macro (`debug_serialize_msg!`) for all sent messages
  + Support `ToMessage` in actor-spawn methods - #62
  + `Context::send` now sends `ToMessage` objects and `Context::send_message` sends `Box<Message>` - #66
  + `Message` support for `ActorAddress` so they can be shared between actors - #67

## 0.1.2 through 0.2.0

  + Added support for spawning actors (`0.1.2`)
  + Introduced runtime manager and round-robin spawn strategy (`0.1.3`)
  + Added configuration support for creating new ActorSystem (`0.2.0`)
  + Implemented `ActorCell` (`0.2.0`)
  + Implemented `ActorAddress` (for local routing only) and address resolution (`0.2.0`)
  + Basic message sending (`0.2.0`)
  + Pattern matching on received types (by use of `Any`) added. Uses newly (`0.2.0`)
    added `Message` type (which subsumes `prost::Message`) to implement `as_any()`. (`0.2.0`)
  + Added `busan-derive` with support for `#[derive(busan::Message)]` proc-macro (`0.2.0`)
  + Updated `hello_world` example to use derive macro in `build.rs` (`0.2.0`)

## 0.1.1

  + Added a CHANGELOG.md file :-)
  + New decision log entry: [000 - Busan][dl_000]
  + `thread_executor` moved into `executor` sub-module
  + Added `shutdown` and `await_shutdown` to `ActorSystem`

  [dl_000]: http://github.com/JohnMurray/busan/blob/master/decisions/000-busan.md
