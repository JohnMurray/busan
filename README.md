# busan (부산)

[![Crates.io][crates-badge]][crates-url]
[![Build Status][actions-badge]][actions-url]
[![GNU GPL v3 licensed][gpl-badge]][gpl-url]
[![Decision log][decision-log-badge]][decision-log-url]
[![Change Log][change-log-badge]][change-log-url]

[crates-badge]: https://img.shields.io/crates/v/busan.svg
[crates-url]: https://crates.io/crates/busan
[actions-badge]: https://github.com/JohnMurray/busan/actions/workflows/ci.yaml/badge.svg
[actions-url]: https://github.com/JohnMurray/busan/actions/workflows/ci.yaml
[gpl-badge]: https://img.shields.io/badge/license-GPL-blue.svg
[gpl-url]: https://github.com/JohnMurray/busan/blob/main/LICENSE
[decision-log-badge]: https://img.shields.io/badge/%F0%9F%93%83-decision%20log-blue
[decision-log-url]: https://github.com/JohnMurray/busan/tree/main/decisions
[change-log-badge]: https://img.shields.io/badge/%F0%9F%93%83-change%20log-blue
[change-log-url]: https://github.com/JohnMurray/busan/blob/main/CHANGELOG.md

Busan is an [actor][wikipedia_actor] implementation for Rust that is currently under heavy
development and is experimental in nature. It is not yet ready for production use, although
it usable to build hobby/personal projects on (with a lot of effort).

[wikipedia_actor]: https://en.wikipedia.org/wiki/Actor_model

## Documentation

The project lacks comprehensive documentation or getting started guides at this time. The best
source of documentation will be found in the rustdocs at [docs.rs][docs-rs-busan] and in the
[examples folder][examples-url].

If you are interested in additional context behind the project or technical decisions, then the
[decision logs][decision-log-url] may be of particular interest.

[docs-rs-busan]: https://docs.rs/busan/latest/busan/
[examples-url]: https://github.com/JohnMurray/busan/blob/main/examples/

### Examples

You can run the examples by specifying the workspace (directory) name, prefixed with `examples_`.

```shell
# Run the current examples
$ cargo run -p examples_ping_pong
$ cargo run -p examples_hello_world

# Trick the cargo run command into listing our example workspaces
$ cargo run -p 2>&1 | grep 'examples_'
```

## Roadmap

The roadmap is constantly evolving, so I don't expect plans to be super detailed outside
the short-term milestones. I'm currently tracking my work in [NOTES.md][notes] and is
constantly evolving.

- ~~[`0.2.0`][m1] - Spawn actors, send and receive messages~~ (shipped)
- `0.3.0` - Core features - lifecycle management, actor/work scheduler, etc.
- `0.4.0` - Ergonomics & Utilities (routers, timers, behaviors, etc)
- `0.4.0` - Observability, test support, docs

[m1]: https://github.com/JohnMurray/busan/milestone/1
[notes]: https://github.com/JohnMurray/busan/blob/main/NOTES.md

Beyond this, I don't have any defined plans. Things on my mind include:

- Remote facilities - remote routing/messaging, clustering, remote actor spawning, etc.
- gRPC bridging (exposing a gRPC interface to communicate with actors)
- Network bridging - a generic take on gRPC bridging that allows for arbitrary network protocols
- DSL for one-off actor systems
- State snapshotting/journaling, actor migration
- Async IO and/or async/await support and/or Tower integration

It's not clear how quickly progress will be made against these milestones and ideas as this is
also a personal experiment in how I think about and manage my open-source projects.

## Contributing

I'm not currently considering code contributions at the moment as the project is still in its
infancy, and I'm still working out the design. However, I am open to suggestions and feedback. If
you have any ideas or suggestions, please start a discussion. I'd also be interested in hearing
about real-world use-cases that are not well-supported by other Rust-based actor implementations.

## Licensing

The project is currently licensed under the [GNU GPL v3][license] license. My intention is to
ensure all early work and development on this project is kept open. I may revisit this if there
is significant commercial interest.

[license]: https://github.com/JohnMurray/busan/blob/main/LICENSE
