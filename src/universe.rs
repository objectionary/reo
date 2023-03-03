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
use std::collections::{HashMap, HashSet};

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
        let v1 = Self::fnd(uni, v, a)?;
        Ok(format!("ν{v1}"))
    }

    /// Find.
    fn fnd(uni: &mut Universe, v: u32, a: &str) -> Result<u32> {
        let v1 = Self::dd(uni, v)?;
        trace!("#fnd(ν{v}, {a}): dd(ν{v}) returned ν{v1}");
        let to = Self::pf(uni, v1, a)?;
        trace!("#fnd(ν{v}, {a}): pf(ν{v}, {a}) returned ν{to}");
        Ok(to)
    }

    /// Path find.
    fn pf(uni: &mut Universe, v: u32, a: &str) -> Result<u32> {
        trace!("#pf(ν{v}, {a}): entering...");
        if let Some(to) = uni.g.kid(v, a) {
            Ok(to)
        } else if let Some(to) = uni.g.kid(v, "ε") {
            Self::fnd(uni, to, a)
        } else if let Some(lv) = uni.g.kid(v, "λ") {
            let lambda = uni.g.data(lv)?.to_utf8()?;
            trace!("#re: calling ν{v}.λ⇓{lambda}(ξ=ν?)...");
            let to = uni.atoms.get(lambda.as_str()).unwrap()(uni, v)?;
            trace!("#re: ν{v}.λ⇓{lambda}(ξ=ν?) returned ν{to}");
            Self::fnd(uni, to, a)
        } else if let Some(to) = uni.g.kid(v, "φ") {
            Self::fnd(uni, to, a)
        } else {
            return Err(anyhow!("There is no way to get .{a} from ν{v}"));
        }
    }

    /// Dynamic dispatch.
    fn dd(uni: &mut Universe, v: u32) -> Result<u32> {
        trace!("#dd(ν{v}): entering...");
        if let Some(to) = uni.g.kid(v, "ε") {
            Self::dd(uni, to)
        } else if let Some(to) = uni.g.kid(v, "β") {
            let a = uni
                .g
                .kids(v)?
                .iter()
                .find(|e| e.0 != "β")
                .unwrap()
                .clone()
                .0;
            let nv = Self::fnd(uni, to, a.as_str())?;
            Self::dd(uni, nv)
        } else if let Some(to) = uni.g.kid(v, "π") {
            let nv = Self::dd(uni, to)?;
            Self::apply(uni, nv, v)
        } else {
            Ok(v)
        }
    }

    /// Apply `v1` to `v2` and return a new vertex.
    fn apply(uni: &mut Universe, v1: u32, v2: u32) -> Result<u32> {
        trace!("#apply(ν{v1}, ν{v2}): entering...");
        let nv = uni.add();
        Self::pull(uni, nv, v1, vec!["σ"].into_iter().collect())?;
        for (a, k) in uni.g.kids(v2)?.into_iter() {
            if a == "π" {
                continue;
            }
            if let Some(t) = uni.g.kid(nv, a.as_str()) {
                if a != "ρ" {
                    return Err(anyhow!(
                        "Can't overwrite ν{v1}.{a}, it already points to ν{t}"
                    ));
                }
            }
            uni.g.bind(nv, k, a.as_str())?;
        }
        trace!("#apply(ν{v1}, ν{v2}): copy ν{v1}+ν{v2} created as ν{nv}");
        Ok(nv)
    }

    /// Pull into `v1` from `v2`.
    fn pull(uni: &mut Universe, v1: u32, v2: u32, skip: HashSet<&str>) -> Result<()> {
        let mut edges = 0;
        for (a, k) in uni.g.kids(v2)?.into_iter() {
            if skip.contains(a.as_str()) {
                continue;
            }
            if let Some(t) = uni.g.kid(v1, a.as_str()) {
                if a != "ρ" {
                    return Err(anyhow!(
                        "Can't overwrite ν{v1}.{a}, it already points to ν{t}"
                    ));
                }
            }
            Self::link(uni, v1, k, a)?;
            edges += 1;
        }
        trace!("#pull(ν{v1}, ν{v2}): pulled {edges} edges");
        Ok(())
    }

    /// Link.
    fn link(uni: &mut Universe, v1: u32, v2: u32, a: String) -> Result<()> {
        if a == "λ" || a == "Δ" || a == "ρ" {
            uni.g.bind(v1, v2, a.as_str())?;
        } else {
            let nv = uni.add();
            uni.g.bind(v1, nv, a.as_str())?;
            uni.g.bind(nv, v1, "ρ")?;
            uni.g.bind(nv, v2, "π")?;
        };
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
fn quick_tests() -> Result<()> {
    for path in sodg_scripts_in_dir("quick-tests") {
        trace!("#quick_tests: {path}");
        let mut s = Script::from_str(fs::read_to_string(path.clone())?.as_str());
        let mut g = Sodg::empty();
        s.deploy_to(&mut g)?;
        trace!("{}", g.clone().to_dot());
        let mut uni = Universe::from_graph(g);
        uni.register("inc", inc);
        uni.register("times", times);
        let hex = uni.dataize("Φ.foo").context(anyhow!("Failure in {path}"))?;
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
