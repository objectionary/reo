<img src="https://www.yegor256.com/images/books/elegant-objects/cactus.svg" height="100px" />

[![EO principles respected here](https://www.elegantobjects.org/badge.svg)](https://www.elegantobjects.org)
[![We recommend IntelliJ IDEA](https://www.elegantobjects.org/intellij-idea.svg)](https://www.jetbrains.com/idea/)

[![cargo](https://github.com/objectionary/reo/actions/workflows/cargo.yml/badge.svg)](https://github.com/objectionary/reo/actions/workflows/cargo.yml)
[![crates.io](https://img.shields.io/crates/v/reo.svg)](https://crates.io/crates/reo)
[![PDD status](http://www.0pdd.com/svg?name=objectionary/reo)](http://www.0pdd.com/p?name=objectionary/reo)
[![Hits-of-Code](https://hitsofcode.com/github/objectionary/reo)](https://hitsofcode.com/view/github/objectionary/reo)
![Lines of code](https://img.shields.io/tokei/lines/github/objectionary/reo)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](https://github.com/objectionary/reo/blob/master/LICENSE.txt)

It's an experimental transpiler of
[EO](https://www.eolang.org) programs to Rust functions.

To build it, install [Rust](https://www.rust-lang.org/tools/install) and then:

```bash
$ cargo build -vv --release
```

If everything goes well, an executable binary will be in `target/release/reo`:

```bash
$ target/release/reo --help
```
