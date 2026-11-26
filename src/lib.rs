// SPDX-FileCopyrightText: Copyright (c) 2022-2026 Yegor Bugayenko
// SPDX-License-Identifier: MIT

//! This is an experimental [SODG](https://github.com/objectionary/sodg)-based
//! virtual machine for
//! [EO](https://www.eolang.org) programs:
//!
//! ```
//! use sodg::Hex;
//! use reo::Universe;
//! let mut uni = Universe::empty();
//! let root = uni.formation(&["foo"]).unwrap();
//! let foo = uni.formation(&[]).unwrap();
//! uni.delta(foo, &Hex::from(42_i64)).unwrap();
//! uni.put(root, "foo", foo).unwrap();
//! assert_eq!(42, uni.dataize("Φ.foo").unwrap().to_i64().unwrap());
//! ```
//!
//! A graph may also be deployed from the output of
//! [`phino sodg`](https://github.com/objectionary/phino):
//!
//! ```
//! use reo::Universe;
//! let g = reo::deploy(
//!     "F c0 foo\nF c1 x φ\nD c2 c1 x\nP c1 φ c2\nF c3\n\
//!      Δ c3 00-00-00-00-00-00-00-2A\nA c4 c1 x c3\nP c0 foo c4",
//! ).unwrap();
//! let mut uni = Universe::from_graph(g).unwrap();
//! assert_eq!(42, uni.dataize("Φ.foo").unwrap().to_i64().unwrap());
//! ```

#![doc(html_root_url = "https://docs.rs/reo/0.0.0")]
#![deny(warnings)]

mod graph;
pub mod org;
mod scripts;
mod universe;

pub use graph::{deploy, merge, validate, Graph, EDGES};

use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;

/// A single atom to be attached to a cell.
///
/// It is a function that is called by [`Universe`] when it needs
/// an attribute or the data of a formation with a λ-asset.
/// The first argument provided is the [`Universe`] itself, while
/// the second one is the ID of the formation with the λ-asset.
/// The atom returns the ID of a cell, usually a fresh formation
/// with a Δ-asset.
pub type Atom = fn(&mut Universe, c: usize) -> Result<usize>;

/// A Universe.
pub struct Universe {
    /// The graph.
    g: Graph,
    /// The capacity of the graph.
    cap: usize,
    /// The ID of the next cell to make.
    next: usize,
    /// All known atoms.
    atoms: HashMap<String, Atom>,
    /// The depth of recursion of the current dataization.
    depth: usize,
    /// Location of snapshots directory.
    snapshots: Option<PathBuf>,
}

#[cfg(test)]
use simple_logger::SimpleLogger;

#[cfg(test)]
use log::LevelFilter;

#[cfg(test)]
#[ctor::ctor]
fn init() {
    SimpleLogger::new()
        .without_timestamps()
        .with_level(LevelFilter::Trace)
        .init()
        .unwrap();
}
