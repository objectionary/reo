<img alt="logo" src="https://www.yegor256.com/images/books/elegant-objects/cactus.svg" height="100px" />

[![EO principles respected here](https://www.elegantobjects.org/badge.svg)](https://www.elegantobjects.org)
[![We recommend IntelliJ IDEA](https://www.elegantobjects.org/intellij-idea.svg)](https://www.jetbrains.com/idea/)

[![cargo](https://github.com/objectionary/reo/actions/workflows/cargo.yml/badge.svg)](https://github.com/objectionary/reo/actions/workflows/cargo.yml)
[![crates.io](https://img.shields.io/crates/v/reo.svg)](https://crates.io/crates/reo)
[![PDD status](http://www.0pdd.com/svg?name=objectionary/reo)](http://www.0pdd.com/p?name=objectionary/reo)
[![codecov](https://codecov.io/gh/objectionary/reo/branch/master/graph/badge.svg)](https://codecov.io/gh/objectionary/reo)
[![Hits-of-Code](https://hitsofcode.com/github/objectionary/reo)](https://hitsofcode.com/view/github/objectionary/reo)
![Lines of code](https://img.shields.io/tokei/lines/github/objectionary/reo)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](https://github.com/objectionary/reo/blob/master/LICENSE.txt)

**ATTENTION**: It's a very early draft currently in active development!
Most probably it doesn't work. Don't try to contribute, unless you know
what you are doing.

It's an experimental transpiler of
[EO](https://www.eolang.org) programs to Rust functions.

First, install
[Rust](https://www.rust-lang.org/tools/install),
[npm](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm),
[Java SE](https://www.oracle.com/java/technologies/downloads/),
and [eolang](https://www.npmjs.com/package/eolang) package.
Then, install `reo` package:

```bash
$ cargo install reo
```

Then, create a simple EO program in `app.eo` file:

```
[] > app
  QQ.io.stdout > @
    "Hello, world!\n"
```

Then, compile it to GMI using [eoc](https://github.com/objectionary/eoc):

```
$ eoc gmi
```

Finally, run it:

```
$ reo --eoc dataize app
```

You should see the "Hello, world!" being printed out.

## How to Contribute

First, install [Rust](https://www.rust-lang.org/tools/install) and then:

```bash
$ cargo test -vv --release
```

If everything goes well, an executable binary will be in `target/release/reo`:

```bash
$ target/release/reo --help
```

Then, fork repository, make changes, send us a [pull request](https://www.yegor256.com/2014/04/15/github-guidelines.html).
We will review your changes and apply them to the `master` branch shortly,
provided they don't violate our quality standards. To avoid frustration,
before sending us your pull request please run `cargo test` again.
