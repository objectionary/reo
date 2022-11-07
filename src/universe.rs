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

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use sodg::Hex;
use sodg::Sodg;
use crate::{Atom, Universe};

impl Universe {
    /// Makes an empty Universe.
    pub fn empty() -> Self {
        let mut g = Sodg::empty();
        g.alert_on(|g, vx| {
            let mut errors = Vec::new();
            for v in vx.iter() {
                let attrs = g.kids(*v).unwrap().iter()
                    .filter(|(a, _, _)| a == "π" || a == "φ")
                    .count();
                if attrs > 1 {
                    errors.push(format!("ν{v} can't have both π and φ"));
                }
            }
            errors
        });
        Universe {
            g,
            atoms: HashMap::new(),
        }
    }

    /// Registers a new atom.
    pub fn register(&mut self, name: &str, a: Atom) {
        self.atoms.insert(name.to_string(), a);
    }

    /// Add new vertex and return its ID.
    pub fn add(&mut self) -> u32 {
        let v = self.g.next_id();
        self.g.add(v).unwrap();
        v
    }

    /// Bind two new vertices.
    pub fn bind(&mut self, v1: u32, v2: u32, a: &str) {
        self.g.bind(v1, v2, a).unwrap();
    }

    /// Save data into a vertex.
    pub fn put(&mut self, v: u32, d: Hex) {
        self.g.put(v, d).unwrap();
    }

    /// Get data.
    pub fn data(&mut self, v: u32) -> Hex {
        self.g.data(v).unwrap().tail(1)
    }

    /// Get lambda.
    pub fn lambda(&mut self, v: u32) -> String {
        self.g.data(v).unwrap().tail(1).to_string()
    }

    /// Has data.
    pub fn has_data(&mut self, v: u32) -> bool {
        let d = self.g.data(v).unwrap();
        !d.is_empty() && d.byte_at(0) == 0x01
    }

    /// Has lambda.
    pub fn has_lambda(&mut self, v: u32) -> bool {
        let d = self.g.data(v).unwrap();
        !d.is_empty() && d.byte_at(0) == 0x02
    }
}

#[cfg(test)]
use anyhow::Result;

#[test]
fn pi_and_phi_together() -> Result<()> {
    let mut uni = Universe::empty();
    let v1 = uni.add();
    let v2 = uni.add();
    uni.bind(v1, v2, "π");
    uni.bind(v1, v2, "φ");
    Ok(())
}

#[cfg(test)]
fn rand(uni: &mut Universe, _: u32) -> Result<u32> {
    let v = uni.add();
    uni.bind(v, 0, "π/int");
    uni.put(v, Hex::from_i64(rand::random::<i64>()));
    Ok(v)
}

#[test]
fn generates_random_int() -> Result<()> {
    let mut uni = Universe::empty();
    let v1 = uni.add();
    uni.bind(0, v1, "int");
    let v2 = uni.add();
    uni.bind(0, v2, "rand");
    uni.bind(0, v2, "x");
    uni.register("rand", rand);
    uni.put(v2, Hex::from_str("rand"));
    let first = uni.dataize("Φ.x.Δ")?.to_i64()?;
    let second = uni.dataize("Φ.x.Δ")?.to_i64()?;
    assert_ne!(first, second);
    Ok(())
}
