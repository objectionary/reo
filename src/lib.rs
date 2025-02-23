// SPDX-FileCopyrightText: Copyright (c) 2022-2025 Yegor Bugayenko
// SPDX-License-Identifier: MIT

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
//! let v2 = uni.add();
//! uni.bind(v1, v2, "Δ");
//! uni.put(v2, Hex::from(42));
//! assert_eq!(42, uni.dataize("Φ.foo").unwrap().to_i64().unwrap());
//! ```

#![doc(html_root_url = "https://docs.rs/reo/0.0.0")]
#![deny(warnings)]

pub mod org;
mod scripts;
mod universe;

use anyhow::Result;
use std::collections::HashMap;

/// A single atom to be attached to a vertex.
///
/// It is a function that is called by [`Universe`] when it's impossible
/// to get data from a vertex. The first argument provided is
/// the [`Universe`] itself, while the second one is the ID of the
/// vertex where the dataization is standing at the moment.
pub type Atom = fn(&mut Universe, v: u32) -> Result<u32>;

/// A Universe.
pub struct Universe {
    /// The graph.
    g: Sodg,
    /// All known atoms.
    atoms: HashMap<String, Atom>,
    /// The depth of recursion of the current dataization.
    depth: usize,
    /// Location of snapshots directory.
    snapshots: Option<String>,
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
