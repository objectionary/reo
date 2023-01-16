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

use crate::{Atom, Universe};
use anyhow::{anyhow, Context, Result};
use log::trace;
use sodg::Sodg;
use sodg::{Hex, Relay};
use std::collections::HashMap;

impl Universe {
    /// Makes an empty Universe.
    pub fn empty() -> Self {
        let mut g = Sodg::empty();
        g.alert_on(|g, vx| {
            let mut errors = Vec::new();
            for v in vx.iter() {
                let attrs = g
                    .kids(*v)
                    .unwrap()
                    .iter()
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
        let v = self.find(format!("{loc}.Δ").as_str())
            .context(format!("Can't find {loc}"))?;
        let data = self
            .g
            .data(v)
            .context(format!("There is no data in ν{v}"))?;
        trace!(
            "#dataize: data found in ν{v} ({} bytes): {}",
            data.len(), data
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
        let v = self
            .g
            .find(0, loc, self)
            .context(format!("Failed to find {loc}"))?;
        Ok(v)
    }
}

impl Relay for Universe {
    /// Resolve a locator on a vertex, if it is not found.
    fn re(&self, at: u32, a: &str) -> Result<String> {
        unsafe {
            let cp = self as *const Self;
            let mp = cp as *mut Self;
            let uni = &mut *mp;
            Self::mut_re(uni, at, a)
        }
    }
}

impl Universe {
    fn relink(uni: &mut Universe, to: u32, loc: String) -> Result<u32> {
        let v = if loc.starts_with('.') {
            uni.find(format!("ν{to}{}", loc).as_str())?
        } else {
            to
        };
        Ok(v)
    }

    /// Resolve a locator on a vertex, if it's not found.
    fn mut_re(uni: &mut Universe, at: u32, a: &str) -> Result<String> {
        trace!("#re(ν{at}.{a}): starting...");
        let found = if let Some((lv, _)) = uni.g.kid(at, "λ") {
            let lambda = uni.g.data(lv)?.to_utf8()?;
            trace!("#re: calling ν{at}.λ⇓{lambda}(ξ=ν?)...");
            let to = uni.atoms.get(lambda.as_str()).unwrap()(uni, at)?;
            trace!("#re: ν{at}.λ⇓{lambda}(ξ=ν?) returned ν{to}");
            format!("ν{to}")
        } else if a == "ξ" || a == "$" {
            format!("ν{at}")
        } else if a == "Φ" || a == "Q" {
            "ν0".to_string()
        } else if a == "Δ" && uni.g.full(at).unwrap() {
            format!("ν{at}")
        } else if let Some((to, loc)) = uni.g.kid(at, "ξ") {
            let t = Self::relink(uni, to, loc)?;
            format!("ν{t}.{a}")
        } else if let Some((to, loc)) = uni.g.kid(at, "π") {
            let t = Self::relink(uni, to, loc)?;
            if let Some((kid, _)) = uni.g.kid(t, "λ") {
                if uni.g.kid(at, "λ").is_none() {
                    let v = uni.add();
                    uni.bind(at, kid, "λ");
                    let lambda = uni.data(kid);
                    uni.put(v, lambda.clone());
                    trace!("#re: ν{at}.π.λ->ν{kid} copied to ν{v} ({lambda}))");
                }
                format!("ν{at}.{a}")
            } else {
                let v = if a == "Δ" {
                    t
                } else {
                    let v = uni.add();
                    uni.bind(v, at, "ρ");
                    if let Some((kid, _)) = uni.g.kid(t, a) {
                        uni.bind(v, kid, "π");
                        trace!("#re: ν{at}.π.{a}->ν{kid} copied to ν{v}");
                    } else {
                        uni.bind(v, t, format!("π/.{a}").as_str());
                    }
                    trace!("#re: ν{at}.π.{a} -> ν{t}");
                    v
                };
                format!("ν{v}")
            }
        } else if let Some((to, loc)) = uni.g.kid(at, "φ") {
            let t = Self::relink(uni, to, loc)?;
            trace!("#re: ν{at}.φ -> ν{t} (ν{at}.{a} not found)");
            format!("ν{t}")
        } else {
            return Err(anyhow!("There is no way to get .{a} from ν{at}"))
        };
        trace!("#re(ν{at}.{a}): found '{found}'");
        Ok(found)
    }
}

#[cfg(test)]
use sodg::Script;

#[cfg(test)]
use std::fs;

#[cfg(test)]
use glob::glob;

#[cfg(test)]
fn rand(uni: &mut Universe, _: u32) -> Result<u32> {
    let v = uni.add();
    uni.bind(v, 0, "π/int");
    uni.put(v, Hex::from(rand::random::<i64>()));
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
    uni.put(lambda, Hex::from_str_bytes("rand"));
    let first = uni.dataize("Φ.x")?.to_i64()?;
    let second = uni.dataize("Φ.x")?.to_i64()?;
    assert_ne!(first, second);
    Ok(())
}

#[cfg(test)]
fn inc(uni: &mut Universe, v: u32) -> Result<u32> {
    let rho = uni.dataize(format!("ν{v}.ρ").as_str())?.to_i64()?;
    let v1 = uni.add();
    uni.put(v1, Hex::from(rho + 1));
    Ok(v1)
}

#[test]
fn quick_tests() -> Result<()> {
    for f in glob("quick-tests/**/*.sodg")? {
        let p = f?;
        let path = p.into_os_string().into_string().unwrap();
        if path.contains('_') {
            continue;
        }
        trace!("#quick_tests: {path}");
        let mut s = Script::from_str(fs::read_to_string(path)?.as_str());
        let mut g = Sodg::empty();
        s.deploy_to(&mut g)?;
        let mut uni = Universe::from_graph(g);
        uni.register("inc", inc);
        assert_eq!(42, uni.dataize("Φ.foo")?.to_i64()?);
    }
    Ok(())
}
