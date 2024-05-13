## Focus Areas

Parts of Busan that need some focused development efforts to get somewhere to
something usable.

- [ ] Actor Supervision
- [ ] Lifecycle Management (pre-condition to supervision)
    - [x] shutdown trees
    - [ ] death-watch / poison-pill
- [x] ACK messages (pre-condition to various forms of message routing)
- [ ] Message routing
- [ ] Stats & Observability
- [ ] Behavior system


### Randome cleanup

- [ ] Document send methods in `actor.rs`
- [ ] Is the resolution step persisted on addresses on copy/move?
- [x] Block on child spawn - the actor should be allocated... I think
- [ ] Do not allow for "dangling" actors created after shutdown has been started
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
- [ ] Refactor the shutdown logic into a separate shutdown manager


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
