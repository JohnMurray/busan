# busan (부산)

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

Busan is an [actor][wikipedia_actor] implementation that is currently under heavy
development and is experimental in nature. It is not currently usable.

[wikipedia_actor]: https://en.wikipedia.org/wiki/Actor_model

## Documentation

The project lacks any real documentation at this time other than reading through
the source code. The project will have better documentation once it reaches an
MVP state.

If you are interested in additional context behind the project or technical
decisions, then the [decision logs][decision-log-url] may be of particular
interest.


<!--

 // Keeping this section as a reminder that I should really have some examples
 // when I can

### Examples

You can run the examples by specifying the workspace (directory) name, prefixed with `examples_`.

```shell
# Run the current examples
$ cargo run -p examples_ping_pong
$ cargo run -p examples_hello_world

# Trick the cargo run command into listing our example workspaces
$ cargo run -p 2>&1 | grep 'examples_'
```

-->

## Licensing

The project is currently licensed under the [GNU GPL v3][license] license because
this is just a personal project that I don't intend to make any money on.

[license]: https://github.com/JohnMurray/busan/blob/main/LICENSE
