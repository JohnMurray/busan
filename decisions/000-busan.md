# Decision 000: Busan
__2022-12-26__

## Context/Problem

The actor model is a semi well-known model for concurrency that remains rather niche in
implementation. Usage of the model seems to center around languages or comprehensive
frameworks (e.g. Erlang/Elixer & Akka). However these langauges, frameworks, and
communities exist because the Actor Model remains a useful abstraction for certain
classes of problems and modeling of distributed systems. That being said, Rust doesn't
have a mature or complete actor model for use. Although some options do exist that
are worth enumerating:

  + Actix - only actively maintined library
  + Acteur
  + Axiom
  + Riker
  + _many_ partially completed, outdated attempts

The only one actively maintained today is Actix and a stroll though the GitHub would
seem to indicate it is only in maintenance mode to support the web-framework. All
of the frameworks are fairly minimal and would require significant development and
investment to get to the level of Erlang/Elixer (especially with OTP) or Akka.

A developer writing Rust wishing to build a system with similar quality and features
to a system built on Akka or OTP must either implement a significant amount of 
functionality, or switch languages to get access to those features.

__Additional Context__ \
I'm not going to go into depth on _what_ the actor model is since there is already a
lot of great information out there.

  + [Wikipedia: Actor Model][wikipedia]
  + [Intro to ActorModel Tech-Talk using Akka][tech-talk] _(shameless plug)_

  [wikipedia]: https://en.wikipedia.org/wiki/Actor_model
  [tech-talk]: https://www.youtube.com/watch?v=lPTqcecwkJg

## Options

### Do nothing

Doing nothing is always a valid option. Actor implementations have alwayse existed
in niche corners of the programming community and perhaps modern developers don't
need these tools (or don't need them often enough to invest in a Rust implementation).
While there are a _lot_ of enthusiastic _starts_ to creating Actor frameworks on
Crates.io (I'm also [guilty here][romeo]), this is no guarantee that this need/desire
exists beyond a niche group.

  [romeo]: https://crates.io/crates/romeo

### Contribute to Actix

Actix is already a popular and established framework with a solid foundation. OSS only
gets better when folks come together to build cool stuff. This is a good choice, a
reasonable choice, a sane choice.

Even though Actix does not have all the facilities of what I would consider a _mature_
actor implementaiton, it _does_ have the basics and a decent community surrounding the
project. This _could_ allow for a focus on only the "interesting bits" given the
foundation that already exists. This could be true leverage for getting the Rust
community to a mature actor framework quickly.

### Build Your Own!!

Almost __always__ the wrong choice. This is hard, thankless, and painstaking work. For
a truly ambitious project, this is the most difficult option. Who likes doing difficult
things? Sane people pick low-effort, high-reward work.

But... None of the actor frameworks really stick close to the true fundamentals of actors
and most aim to "improve" on it in some way or another. (Writing messages in req/resp
pairs, typing messages, pub/sub, etc) But ya know what? Erlang and Akka have their
roots in a more "pure" actor implementation. Of course they've built a bunch of
useful features on from there (OTP, typed actors, journaling, etc). But the basics of
the actor system are what make it so powerful and empowers a lot of functionality to be
built from there, this shouldn't be forgotten.

Building from scratch would allow the framework to adhere to more of these "pure" actor
principles.

## Choice

Yeah... I'm going to build my own. I can do this, right? RIGHT? I mean, I'll just stat
with some basic actors, write a scheduler, decide on a distribution strategy. Wait...
speaking of strategies I also need some sort of spawn strategy for balancing actors on
threads. Wait... do I need threads? What about async IO stuff :thinking:

_a few moments later_

![I think I got it](https://github.com/JohnMurray/busan/blob/main/decisions/assets/pepe-silvia.jpg)


Joking aside, I think creating the solution I'm envisioning _does require_ starting from
a blank slate. There are some fundamental decisions that most actor implementation start
out making that I think should be removed and there are some (possibly controversial)
decisions that I'd like to experiment with that are not fit for an existing project of
any real maturity.

This will become more clear as this project evolves and more decision logs are written,
but a short preview (without context) might be:

  + All messages are represented via Protobuf
  + Actors are "untyped" that (in essence) they pattern match on the incoming type
  + By default actors are not required to respond to a message
  + _Every_ value sent via a message must be serialized

There are probably a few things I'm forgetting as I'm writing this and these sound like
odd objectives without more context (which will come later).

It's worth acknowledging that this is __the most risky__ option I could choose. Doing
nothing would be the safest, because I could have declared victory before even writing
this decision log. Contributing to Actix would be the quickest path completion and
value for the community. Building my own is risky for a lot of reasons; I could burn
out or get tired, I could build something no one wants whereas building on Actix would
give me quicker feedback, I could take so long that some future _thing_ makes my work
redundant.
