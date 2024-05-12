## Focus Areas

Parts of Busan that need some focused development efforts to get somewhere to
something usable.

- [ ] Actor Supervision
- [ ] Lifecycle Management (pre-condition to supervision)
- [ ] ACK messages (pre-condition to various forms of message routing)
- [ ] Message routing
- [ ] Stats & Observability
- [ ] Behavior system


### Actor Shutdown

This is pretty broken right now and a hack. We should do this properly.

- [ ] Initial shutdown request
    - [ ] Block receipt of new messages
    - [ ] Send shutdown notices to all watchers
    - [ ] Call `before_shutdown` handle on the actor
    - [ ] Call shutdown (__and wait for completion__) on all children
    - [ ] Call `after_shutdown` handle on the actor
- [ ] Shutdown should be handled by the runtime-manager
- [ ] Shutdown actions should be designed in terms of trees and sub-trees

### Message ACK'nowledgement

- [x] Track the nonce-state so that it can be auto-incremented without user tracking
- [x] Send the nonce from the actor context
- [x] Read the nonce-value in an actor-specific method that is user-overridable
- [x] Example
- [x] Revise example with working spawn
- [ ] Get example working wit proper shutdown


### Randome cleanup

- [ ] Document send methods in `actor.rs`
- [ ] Is the resolution step persisted on addresses on copy/move?
- [x] Block on child spawn - the actor should be allocated... I think
- [ ] crossbeam channel `send` macro that asserts sending, something like:
    ```rust
    macro_rules! debug_assert_send {
        let result = channel.send(...)
        debug_assert!(result.is_ok(), "Failed to send along channel...");
        match result {
            Ok(_) => (),
            Err(e) => error!("Failed to send along chanel... {}", e);
        }
    }
    ```


## Offline Development

- `cargo doc --open` to view the rustdocs for all dependencies (_all_)
- `rustup doc` to view the rustdocs for the standard library and other useful
  tools like 'Rust by Example'
- crates with special docs
    - ratatui - see `README.md` for npm instructions to launch the site (git
      lfs already installed)
    - tokio - see `README.md` for instruction to build / run the docs
- little book of macros
    -  `mdbook serve --port 1234 --open`
