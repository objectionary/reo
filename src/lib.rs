// Copyright (c) 2022 Yegor Bugayenko
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included
// in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NON-INFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

//! This is an experimental [SODG](https://github.com/objectionary/sodg)-based
//! virtual machine for
//! [EO](https://www.eolang.org) programs:
//!
//! ```
//! use sodg::Hex;
//! use reo::Universe;
//! let mut uni = Universe::empty();
//! let root = uni.add();
//! let v1 = uni.add();
//! uni.bind(root, v1, "foo");
//! uni.put(v1, Hex::from(42));
//! assert_eq!(42, uni.dataize("Φ.foo").unwrap().to_i64().unwrap());
//! ```

#![doc(html_root_url = "https://docs.rs/reo/0.0.0")]
// #![deny(warnings)]

mod org;
mod scripts;
mod universe;

use anyhow::Result;
use std::collections::HashMap;

/// A single atom.
pub type Atom = fn(&mut Universe, v: u32) -> Result<u32>;

/// A Universe.
pub struct Universe {
    g: Sodg,
    atoms: HashMap<String, Atom>
}

#[cfg(test)]
use simple_logger::SimpleLogger;

#[cfg(test)]
use log::LevelFilter;
use sodg::Sodg;

#[cfg(test)]
#[ctor::ctor]
fn init() {
    SimpleLogger::new()
        .without_timestamps()
        .with_level(LevelFilter::Trace)
        .init()
        .unwrap();
}
