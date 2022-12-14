# busan

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
development and is experimental in nature. It is not yet ready for production use.

  [wikipedia_actor]: https://en.wikipedia.org/wiki/Actor_model

## Documentation

The project lacks comprehensive documentation at this time, however I am experimenting with [decision
logs][decision-log-url] as a way to document and communicate the major design decisions that were
made.

Of course the source code is also lightly documented and available at [docs.rs][docs-rs-busan].

  [docs-rs-busan]: https://docs.rs/busan/latest/busan/

## Roadmap

I don't have a long-term roadmap laid out for the project and I don't expect plans to be super detailed
outside the short-term milestones. I'm currently using GitHub's Project feature to organize my work,
which is publicly viewable [here][github_project]. Generally my plan looks like:

  + [`0.2.0`][m1] - Spawn actors, send and receive messages
  + `0.3.0` - Core features - lifecycle management, actor/work scheduler, etc.
  + `0.4.0` - Actor utilities - routers, timers, ask-pattern, behaviors, etc.

  [m1]: https://github.com/JohnMurray/busan/milestone/1

Beyond this, I don't have any defined plans. Things on my mind include:

  + Remote facilities - remote routing/messaging, clustering, remote actor spawning, etc.
  + gRPC bridging (exposing a gRPC interface to communicate with actors)
  + Network bridging - a generic take on gRPC bridging that allows for arbitrary network protocols
  + DSL for one-off actor systems
  + Observability - tracing, logging, metrics, etc.
  + State snapshotting/journaling, actor migration
  + Async IO and/or async/await support and/or Tower integration

It's not clear how quickly progress will be made against these milestones and ideas as this is
also a personal experiment in how I think and manage my open-source projects.

  [github_project]: https://github.com/users/JohnMurray/projects/1/views/1

## Contributing

I'm not currently considering code contributions at the moment as the project is still in its infancy,
and I'm still working out the design. However, I am open to suggestions and feedback. If you have any
ideas or suggestions, please open an issue for discussion. I'd also be interested in hearing about
real-world use-cases that are not well-supported by other Rust-based actor implementations.
