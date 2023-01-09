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

use std::collections::HashMap;
use std::str::FromStr;
use anyhow::{anyhow, Context, Result};
use log::trace;
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
        Self::from_graph(g)
    }

    /// Makes a Universe from a graph.
    pub fn from_graph(g: Sodg) -> Self {
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

    /// Save data into a vertex. If there is no vertex `v`, the function
    /// will panic.
    pub fn put(&mut self, v: u32, d: Hex) {
        self.g.put(v, d).unwrap();
    }

    /// Get the `Hex` from the vertex.
    /// If there is no vertex `v`, the function will panic.
    pub fn data(&mut self, v: u32) -> Hex {
        self.g.data(v).unwrap()
    }

    /// Dataize by absolute locator. The search always starts from the
    /// root node of the tree. It is recommended to start the locator
    /// from "Φ". If you need to find any vertex starting from non-root
    /// one, use `find` method.
    pub fn dataize(&mut self, loc: &str) -> Result<Hex> {
        let v = self.find(loc)?;
        let data = self.g.data(v)
            .context(format!("There is no data in ν{v}"))?
            .tail(1);
        trace!(
            "#dataize: data found in ν{v} ({} bytes), all good!",
            data.len()
        );
        Ok(data)
    }

    /// Find vertex by absolute locator. The search always starts from the
    /// root node of the tree. It is recommended to start the locator
    /// from "Φ".
    pub fn find(&mut self, loc: &str) -> Result<u32> {
        if self.g.is_empty() {
            return Err(anyhow!("The Universe is empty, can't dataize {loc}"));
        }
        let v = self.g
            .find_with_closure(0, loc, |v, a, b| {
                // return self.resolve(v, a, b);
                return Ok("boom".to_string());
            })
            .context(format!("Failed to find {loc}"))?;
        Ok(v)
    }

    /// Resolve a locator on a vertex, if it's not found.
    fn resolve(&mut self, at: u32, a: &str, b: &str) -> Result<String> {
        trace!("#resolve(ν{at}, '{a}', '{b}'): starting...");
        // if k == "▲" {
        //     xi = xis.pop_back().unwrap();
        //     trace!("#find: ξ loaded to ν{} by ▲", xi);
        //     continue;
        // }
        // if k == "▼" {
        //     xis.push_back(xi);
        //     trace!("#find: ξ=ν{} saved by ▼", xi);
        //     continue;
        // }
        if a.starts_with("ν") {
            let num: String = a.chars().skip(1).collect::<Vec<_>>().into_iter().collect();
            let v = u32::from_str(num.as_str())?;
            // xi = v;
            trace!("#resolve: jumping directly to ν{v}");
            return Ok(format!("ν{v}"));
        }
        if a == "ξ" || a == "$" {
            trace!("#resolve: ν{at}.ξ -> {at}");
            return Ok(format!("ν{at}"));
        }
        if a == "Φ" || a == "Q" {
            // xi = v;
            trace!("#resolve: Φ/ν{at}");
            return Ok("ν0".to_string());
        }
        if let Some(to) = self.g.kid(at, "ξ") {
            trace!("#resolve: ν{at}.ξ -> ν{to} (.{a} not found)");
            // locator.push_front(k);
            return Ok(format!("ν{to}"));
        }
        if let Some(to) = self.g.kid(at, "π") {
            trace!("#resolve: ν{at}.π -> ν{to} (.{a} not found)");
            // locator.push_front(k);
            return Ok(format!("ν{to}"));
        }
        if let Some(to) = self.g.kid(at, "φ") {
            trace!("#resolve: ν{at}.φ -> ν{to} (.{a} not found)");
            // xi = v;
            // locator.push_front(k);
            return Ok(format!("ν{to}"));
        }
        if let Some(lv) = self.g.kid(at, "λ") {
            let lambda = self.data(lv).to_utf8().unwrap();
            trace!("#resolve: at ν{at} calling λ{lambda}(ξ=ν?)...");
            let to = self.atoms.get(lambda.as_str()).unwrap()(self, 0)?;
            // locator.push_front(format!("ν{}", to));
            trace!("#resolve: λ{lambda} in ν{at}(ξ=ν?) returned ν{to}");
            // trace!(
            //     "#find: λ at λ{} reset locator to '{}'",
            //     v,
            //     itertools::join(locator.clone(), ".")
            // );
            return Ok(format!("ν{to}"));
        }
        let others : Vec<String> = self.g.kids(at.clone())?.into_iter().map(|(k, a, b)| k).collect();
        return Err(anyhow!(
            "Can't find .{a} in ν{at} among other {} attribute{}: {}",
            others.len(),
            if others.len() == 1 { "" } else { "s" },
            others.join(", ")
        ));
    }
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
    let root = uni.add();
    assert_eq!(0, root);
    let v1 = uni.add();
    uni.bind(root, v1, "int");
    let v2 = uni.add();
    uni.bind(root, v2, "rand");
    uni.bind(root, v2, "x");
    uni.register("rand", rand);
    let lambda = uni.add();
    uni.bind(v2, lambda, "λ");
    uni.put(lambda, Hex::from_str("rand"));
    let first = uni.dataize("Φ.x.Δ")?.to_i64()?;
    let second = uni.dataize("Φ.x.Δ")?.to_i64()?;
    assert_ne!(first, second);
    Ok(())
}
