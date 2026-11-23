<img alt="logo" src="https://www.objectionary.com/cactus.svg" height="100px" />

[![EO principles respected here](https://www.elegantobjects.org/badge.svg)](https://www.elegantobjects.org)
[![We recommend IntelliJ IDEA](https://www.elegantobjects.org/intellij-idea.svg)](https://www.jetbrains.com/idea/)

[![cargo](https://github.com/objectionary/reo/actions/workflows/cargo.yml/badge.svg)](https://github.com/objectionary/reo/actions/workflows/cargo.yml)
[![crates.io](https://img.shields.io/crates/v/reo.svg)](https://crates.io/crates/reo)
[![PDD status](https://www.0pdd.com/svg?name=objectionary/reo)](https://www.0pdd.com/p?name=objectionary/reo)
[![codecov](https://codecov.io/gh/objectionary/reo/branch/master/graph/badge.svg)](https://codecov.io/gh/objectionary/reo)
[![Hits-of-Code](https://hitsofcode.com/github/objectionary/reo)](https://hitsofcode.com/view/github/objectionary/reo)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](https://github.com/objectionary/reo/blob/master/LICENSE.txt)

> [!WARNING]
> It's a very early draft currently in active development!
> Most probably it doesn't work.
> Don't try to contribute, unless you know what you are doing.

It's an experimental virtual machine of [EO](https://www.eolang.org) programs,
represented as [SODG](https://github.com/objectionary/sodg) graphs.

First, install
[Rust](https://www.rust-lang.org/tools/install)
and [phino](https://github.com/objectionary/phino).
Then, install `reo` package:

```bash
$ cargo install reo
```

Then, write a simple 𝜑-expression in `app.phi` file:

```
⟦
  bar ↦ ⟦ x ↦ ∅, φ ↦ ξ.x ⟧,
  foo ↦ Φ.bar(x ↦ ⟦ Δ ⤍ 00-00-00-00-00-00-00-2A ⟧)
⟧
```

Then, translate it to SODG with `phino` and compile it to a binary file:

```bash
$ phino sodg app.phi > app.sodg
$ reo compile app.sodg app.reo
```

Finally, dataize the `foo` object:

```bash
$ reo dataize app.reo foo
```

You should see `00-00-00-00-00-00-00-2A` printed out, which is 42.

`reo compile` rejects positional attributes, like `α0`.
Use named ones instead.
REO finds atoms by the names in their `λ` assets, like `L_int_plus`.

## How to Contribute

First, install [Rust](https://www.rust-lang.org/tools/install) and then:

```bash
$ cargo test -vv --release
```

Make sure the code is formatted and has no warnings:

```bash
$ cargo fmt --check
$ cargo clippy --all-targets -- -D warnings
```

If everything goes well, an executable binary will be in `target/release/reo`:

```bash
$ target/release/reo --help
```

Then, fork repository, make changes, send us a [pull request](https://www.yegor256.com/2014/04/15/github-guidelines.html).
We will review your changes and apply them to the `master` branch shortly,
provided they don't violate our quality standards. To avoid frustration,
before sending us your pull request please run `cargo test` again.
