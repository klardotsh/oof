# oof: a mostly-declarative system setter-upper

`oof` enables "mostly-declarative" setups of Unix systems that lean heavily on
existing distribution and system tooling, packaging, etc., through standardized
cross-distribution abstractions called _Intents_.

`oof` can't guarantee every byte on the target system's drive will be identical
to another system running the same `oof` recipe. If you need those guarantees,
use [Nix](https://nixos.org), [Guix](https://guix.gnu.org/), or something along
similar lines. However, if you're looking to keep your existing distribution for
whatever reason (maybe you need easy interopability with proprietary software or
other tools that depend on an
[FHS](https://en.wikipedia.org/wiki/Filesystem_Hierarchy_Standard)-compliant
environment, maybe you like the release cycle, or maybe you just like modular
tooling, whatever!) and still want a taste of declarative system management,
then `oof` might be for you!

## Getting Started

_This section reserved, to be filled in eventually..._

## Where can I use it?

`oof` should generally run on any system for which an `oof` backend and Rust
1.51+ compiler exists, though the degrees of integration and support vary
greatly between distributions.

### Linux

- `apk`-based distributions, notably Alpine and PostmarketOS
- `pacman`- and `libalpm`-based distributions, notably Arch and Artix

### Others

- `dpkg`-based distribution support, notably Debian and Ubuntu, is planned
- MacOS support via `homebrew` is likely at some point

> oof's automated test suites attempt to ensure quality and stability for any
> systems it supports, but we can't promise we've tested everything on every
> system, so please report any issues you run into!

## Extending or hacking on `oof`

`oof` is implemented in Rust, and intends to prioritize
backwards-compatibility, ease of understanding, ease of use, and extensibility
through external executables where appropriate (in other words: the core engine
should be kept slim). `oof` config files use the
[OVER](https://github.com/m-cat/over) format targeting well-defined,
well-documented, and strictly versioned schemas. Deprecation of a schema should
be publicized well in advance of its removal to reduce end-user churn to every
extent possible. In an ideal world, _`oof` configs could work forever_. It is
our job as developers to limit the headaches we inflict on end-users,
especially for foundational/infrastructural software.

### A word about code style

`oof`'s implementation and code style should always favor readability,
simplicity, and graceful degradation over philosophical correctness, cleverness,
or other forms of "purity":

- The more readable it is, the more people will feel they can contribute, which
  expands both the breadth of technical scenarios `oof` can handle, as well as
  helps eliminate single points of failure in maintainership.

- The more simple it is, the more likely it is to successfully run on more
  machines, on more distributions, and for more usecases, because simplicity
  lends itself to understandability and hackability.

- Degrading gracefully ensures that the user sees as few gnarly errors as
  possible, and when they do pop up, the user is presented with the tools and
  information necessary to fix the issue, or to ask for help. It hopefully also
  allows them to get _most of the way_ to the system they intended, even if some
  sub-assembly doesn't go to plan.

> "other forms of "purity"" is shorthand for: it shouldn't take a PhD in monad
> theory to get an idea of how `oof` is building your system!

## Copying, Contributing, and Legal

`oof`'s implementation, specification, documentation, artwork, and other assets
are all [Copyfree](http://copyfree.org/), released under the [Creative Commons
Zero 1.0 dedication](https://creativecommons.org/publicdomain/zero/1.0/). Thus,
while upstream contributions are welcomed and encouraged for the benefit of us
all, you are free to use `oof` for any purpose and in any context.

Contributions to `oof`'s first-party repositories must be dedicated under the
same terms. By submitting a contribution to an `oof` project, you assert the
following (this is the [Unlicense waiver](https://unlicense.org/WAIVER)):

> I dedicate any and all copyright interest in this software to the
> public domain. I make this dedication for the benefit of the public at
> large and to the detriment of my heirs and successors. I intend this
> dedication to be an overt act of relinquishment in perpetuity of all
> present and future rights to this software under copyright law.
>
> To the best of my knowledge and belief, my contributions are either
> originally authored by me or are derived from prior works which I have
> verified are also in the public domain and are not subject to claims
> of copyright by other parties.
>
> To the best of my knowledge and belief, no individual, business,
> organization, government, or other entity has any copyright interest
> in my contributions, and I affirm that I will not make contributions
> that are otherwise encumbered.
