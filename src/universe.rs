// Copyright (c) 2022-2023 Yegor Bugayenko
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
use std::path::Path;
use std::str::FromStr;

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
                    .filter(|(a, _)| a == "π" || a == "φ")
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
            depth: 0,
        }
    }

    /// Registers a new atom.
    pub fn register(&mut self, name: &str, a: Atom) {
        self.atoms.insert(name.to_string(), a);
        trace!("#register: atom {name} registered as {:p}", a as *const ());
    }

    /// Add new vertex and return its ID.
    pub fn add(&mut self) -> u32 {
        let v = self.g.next_id();
        self.g
            .add(v)
            .context(anyhow!("Failed to add ν{v}"))
            .unwrap();
        v
    }

    /// Bind two new vertices.
    pub fn bind(&mut self, v1: u32, v2: u32, a: &str) {
        self.g
            .bind(v1, v2, a)
            .context(anyhow!("Failed to bind ν{v1} to ν{v2} as '{a}'"))
            .unwrap();
    }

    /// Save data into a vertex. If there is no vertex `v`, the function
    /// will panic.
    pub fn put(&mut self, v: u32, d: Hex) {
        self.g
            .put(v, d)
            .context(anyhow!("Failed to put the data to ν{v}"))
            .unwrap();
    }

    /// Get the `Hex` from the vertex.
    /// If there is no vertex `v`, the function will panic.
    pub fn data(&mut self, v: u32) -> Hex {
        self.g
            .data(v)
            .context(anyhow!("Failed to get data from ν{v}"))
            .unwrap()
    }

    /// Dataize by absolute locator. The search always starts from the
    /// root node of the tree. It is recommended to start the locator
    /// from "Φ". If you need to find any vertex starting from non-root
    /// one, use `find` method.
    pub fn dataize(&mut self, loc: &str) -> Result<Hex> {
        let v = self
            .find(format!("{loc}.Δ").as_str())
            .context(format!("Can't find {loc}"))?;
        let data = self
            .g
            .data(v)
            .context(format!("There is no data in {}", self.g.v_print(v)))?;
        trace!(
            "#dataize: data found in ν{v} ({} bytes): {}",
            data.len(),
            data
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

    /// Get a slice of the graph by the locator.
    pub fn slice(&mut self, loc: &str) -> Result<Sodg> {
        self.g
            .slice_some(loc, |_v, _to, a| !a.starts_with('ρ') && !a.starts_with('σ'))
    }

    /// Dump the graph to a file.
    pub fn dump(&self, p: &Path) -> Result<usize> {
        self.g.save(p)
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
    /// Resolve a locator on a vertex, if it's not found. The returned string
    /// is treated as a new locator, where the search will continue.
    fn mut_re(uni: &mut Universe, v: u32, a: &str) -> Result<String> {
        if a == "Φ" {
            return Ok("ν0".to_string());
        };
        let v1 = uni.fnd(v, a, 0)?;
        Ok(format!("ν{v1}"))
    }

    /// Find.
    fn fnd(&mut self, v: u32, a: &str, psi: u32) -> Result<u32> {
        self.check_recursion()?;
        let v1 = self.dd(v, psi)?;
        trace!("#fnd(ν{v}, {a}, {psi}): dd(ν{v}) returned ν{v1}");
        let to = self.pf(v1, a, psi)?;
        trace!("#fnd(ν{v}, {a}, {psi}): pf(ν{v}, {a}) returned ν{to}");
        self.depth -= 1;
        Ok(to)
    }

    /// Path find.
    fn pf(&mut self, v: u32, a: &str, psi: u32) -> Result<u32> {
        self.check_recursion()?;
        trace!("#pf(ν{v}, {a}, {psi}): entering...");
        let r = if let Some(to) = self.g.kid(v, a) {
            Ok(to)
        } else if let Some(lv) = self.g.kid(v, "λ") {
            let lambda = self.g.data(lv)?.to_utf8()?;
            trace!("#re: calling ν{v}.λ⇓{lambda}(ξ=ν?)...");
            let to = self
                .atoms
                .get(lambda.as_str())
                .context(anyhow!(
                    "Can't find function {lambda} among {} others",
                    self.atoms.len()
                ))
                .unwrap()(self, v)?;
            trace!("#re: ν{v}.λ⇓{lambda}(ξ=ν?) returned ν{to}");
            self.fnd(to, a, psi)
        } else if let Some(to) = self.g.kid(v, "φ") {
            self.fnd(to, a, psi)
        } else if let Some(to) = self.g.kid(v, "γ") {
            let t = Self::fnd(self, to, a, psi)?;
            self.g.bind(v, t, a)?;
            Ok(t)
        } else {
            Err(anyhow!(
                "There is no way to get .{a} from {}",
                self.g.v_print(v)
            ))
        };
        self.depth -= 1;
        r
    }

    /// Dynamic dispatch.
    fn dd(&mut self, v: u32, psi: u32) -> Result<u32> {
        self.check_recursion()?;
        trace!("#dd(ν{v}, {psi}): entering...");
        let r = if let Some(to) = self.g.kid(v, "ε") {
            self.dd(to, psi)
        } else if let Some(to) = self.g.kid(v, "ξ") {
            if psi == 0 {
                self.dd(to, psi)
            } else {
                self.dd(psi, psi)
            }
        } else if let Some(to) = self.g.kid(v, "β") {
            let a = self
                .g
                .kids(v)?
                .iter()
                .find(|e| e.0 != "β")
                .unwrap()
                .clone()
                .0;
            let nv = self.fnd(to, a.as_str(), psi)?;
            self.dd(nv, psi)
        } else if let Some(to) = self.g.kid(v, "π") {
            let psi2 = if let Some(p) = self.g.kid(v, "ψ") {
                p
            } else {
                v
            };
            let nv = self.dd(to, psi2)?;
            self.apply(nv, v)
        } else {
            Ok(v)
        };
        self.depth -= 1;
        r
    }

    /// Apply `v1` to `v2` and return a new vertex.
    fn apply(&mut self, v1: u32, v2: u32) -> Result<u32> {
        trace!("#apply(ν{v1}, ν{v2}): entering...");
        self.depth += 1;
        let nv = self.g.next_id();
        self.g.add(nv)?;
        self.pull(nv, v1)?;
        self.push(nv, v2)?;
        trace!("#apply(ν{v1}, ν{v2}): copy ν{v1}+ν{v2} created as ν{nv}");
        self.depth -= 1;
        Ok(nv)
    }

    /// Pull into `v1` from `v2`.
    fn pull(&mut self, v1: u32, v2: u32) -> Result<()> {
        for (a, k) in self.g.kids(v2)?.into_iter() {
            if a == "σ" || a == "β" || a == "π" {
                continue;
            }
            self.up(v1, k, a)?;
        }
        Ok(())
    }

    /// Link.
    fn up(&mut self, v1: u32, v2: u32, a: String) -> Result<()> {
        if a == "λ" || a == "Δ" || a == "ρ" || self.nil(v2)? {
            self.g.bind(v1, v2, a.as_str())?;
        } else {
            let nv = self.g.next_id();
            self.g.add(nv)?;
            self.g.bind(v1, nv, a.as_str())?;
            self.g.bind(nv, v1, "ρ")?;
            self.g.bind(nv, v1, "ψ")?;
            self.g.bind(nv, v2, "π")?;
        };
        Ok(())
    }

    /// Push from `v2` to `v1`.
    fn push(&mut self, v1: u32, v2: u32) -> Result<()> {
        for (a, k) in self.g.kids(v2)?.into_iter() {
            if a == "π" {
                continue;
            }
            self.down(v1, k, a)?;
        }
        Ok(())
    }

    /// Link down.
    fn down(&mut self, v1: u32, v2: u32, a: String) -> Result<()> {
        let a1 = self.tie(v1, a)?;
        self.g.bind(v1, v2, a1.as_str())?;
        Ok(())
    }

    /// Tie an existing name with a new name.
    fn tie(&mut self, v: u32, a: String) -> Result<String> {
        if a == "ρ" || a == "ψ" || a == "σ" {
            trace!("#tie(ν{v}, {a}): it's a direct tie");
            return Ok(a);
        }
        if a == "Δ" && self.g.kid(v, a.as_str()).is_none() {
            trace!("#tie(ν{v}, {a}): it's a new data");
            return Ok(a);
        }
        if let Some(v1) = self.g.kid(v, a.as_str()) {
            if self.nil(v1)? {
                trace!("#tie(ν{v}, {a}): it's a nil");
                return Ok(a);
            }
        }
        if a.starts_with('α') {
            let tail: String = a.chars().skip(1).collect::<Vec<_>>().into_iter().collect();
            let i = usize::from_str(tail.as_str())?;
            let a1 = self
                .g
                .kids(v)?
                .into_iter()
                .filter(|(aa, _)| aa.is_ascii())
                .nth(i)
                .unwrap();
            trace!("#tie(ν{v}, {a}): the {i}th attribute is {}", a1.0);
            return self.tie(v, a1.0);
        }
        return Err(anyhow!("Can't tie to ν{v}.{a}"));
    }

    /// The vertex is a dead-end, a nil.
    fn nil(&mut self, v: u32) -> Result<bool> {
        let kids = self.g.kids(v)?;
        return Ok(kids.len() == 1 && kids.iter().all(|(a, _)| a == "ρ"));
    }

    fn check_recursion(&mut self) -> Result<()> {
        self.depth += 1;
        if self.depth > 20 {
            return Err(anyhow!("The recursion is too deep ({} levels)", self.depth));
        }
        Ok(())
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
    let v2 = uni.add();
    uni.bind(v, v2, "Δ");
    uni.put(v2, Hex::from(rand::random::<i64>()));
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
    let v2 = uni.add();
    uni.bind(v1, v2, "Δ");
    uni.put(v2, Hex::from(rho + 1));
    Ok(v1)
}

#[cfg(test)]
fn times(uni: &mut Universe, v: u32) -> Result<u32> {
    let rho = uni.dataize(format!("ν{v}.ρ").as_str())?.to_i64()?;
    let x = uni.dataize(format!("ν{v}.x").as_str())?.to_i64()?;
    let v1 = uni.add();
    let v2 = uni.add();
    uni.bind(v1, v2, "Δ");
    uni.put(v2, Hex::from(rho * x));
    Ok(v1)
}

#[cfg(test)]
fn sodg_scripts_in_dir(dir: &str) -> Vec<String> {
    let mut paths = vec![];
    for f in glob(format!("{dir}/**/*.sodg").as_str()).unwrap() {
        let p = f.unwrap();
        let path = p.into_os_string().into_string().unwrap();
        if path.contains('_') {
            continue;
        }
        paths.push(path);
    }
    paths.sort();
    paths
}

#[test]
fn find_absent_vertex() -> Result<()> {
    let g = Sodg::empty();
    let mut uni = Universe::from_graph(g);
    uni.add();
    assert!(uni.dataize("Φ.foo").is_err());
    Ok(())
}

#[test]
fn fnd_absent_vertex() -> Result<()> {
    let g = Sodg::empty();
    let mut uni = Universe::from_graph(g);
    uni.add();
    assert!(uni.dataize("ν42.foo").is_err());
    Ok(())
}

#[test]
fn quick_tests() -> Result<()> {
    for path in sodg_scripts_in_dir("quick-tests") {
        trace!("#quick_tests: {path}");
        let mut s = Script::from_str(fs::read_to_string(path.clone())?.as_str());
        let mut g = Sodg::empty();
        s.deploy_to(&mut g)?;
        trace!("Before:\n {}", g.clone().to_dot());
        let mut uni = Universe::from_graph(g);
        uni.register("inc", inc);
        uni.register("times", times);
        let r = uni.dataize("Φ.foo");
        trace!("After:\n {}", uni.g.to_dot());
        let hex = r.context(anyhow!("Failure in {path}"))?;
        assert_eq!(42, hex.to_i64()?, "Failure in {path}");
    }
    Ok(())
}

#[test]
fn quick_errors() -> Result<()> {
    for path in sodg_scripts_in_dir("quick-errors") {
        trace!("#quick_errors: {path}");
        let mut s = Script::from_str(fs::read_to_string(path.clone())?.as_str());
        let mut g = Sodg::empty();
        s.deploy_to(&mut g)?;
        let mut uni = Universe::from_graph(g);
        uni.register("inc", inc);
        uni.register("times", times);
        assert!(
            uni.dataize("Φ.foo").is_err(),
            "A failure is expected in {path}, but it didn't happen"
        );
    }
    Ok(())
}
