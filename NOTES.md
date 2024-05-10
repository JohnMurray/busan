## Focus Areas

Parts of Busan that need some focused development efforts to get somewhere to
something usable.

- [ ] Actor Supervision
- [ ] Lifecycle Management (pre-condition to supervision)
- [ ] ACK messages (pre-condition to various forms of message routing)
- [ ] Message routing
- [ ] Stats & Observability
- [ ] Behavior system

### Message ACK'nowledgement

- [x] Track the nonce-state so that it can be auto-incremented without user tracking
- [x] Send the nonce from the actor context
- [x] Read the nonce-value in an actor-specific method that is user-overridable
- [x] Example
- [ ] Revise example with working spawn


### Randome cleanup

- [ ] Document send methods in `actor.rs`
- [ ] Is the resolution step persisted on addresses on copy/move?
- [ ] Block on child spawn - the actor should be allocated... I think


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
