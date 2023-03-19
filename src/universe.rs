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
use std::cmp;
use std::collections::HashMap;
use std::io::prelude::*;
use std::path::Path;
use std::str::FromStr;

macro_rules! enter {
    ($self:expr, $($arg:tt)+) => {
        $self.enter_it(format!($($arg)+))?;
    }
}

macro_rules! exit {
    ($self:expr, $($arg:tt)+) => {
        $self.exit_it(format!($($arg)+))?;
    }
}

// self.depth += 1;
// if self.depth > 20 {
// return Err(anyhow!("The recursion is too deep ({} levels)", self.depth));
// }

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
            .put(v, &d)
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
            .context(format!("There is no data in {}", self.g.v_print(v)?))?;
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
        enter!(self, "#fnd(ν{v}, {a}, {psi}): entered...");
        let v1 = self.dd(v, psi)?;
        let to = self.pf(v1, a, psi)?;
        exit!(self, "#fnd(ν{v}, {a}, {psi}): pf(ν{v}, {a}) returned ν{to}");
        Ok(to)
    }

    /// Path find.
    fn pf(&mut self, v: u32, a: &str, psi: u32) -> Result<u32> {
        enter!(self, "#pf(ν{v}, {a}, {psi}): entering...");
        let r = if let Some(to) = self.g.kid(v, a) {
            to
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
            self.fnd(to, a, psi)?
        } else if let Some(to) = self.g.kid(v, "φ") {
            self.fnd(to, a, psi)?
        } else if let Some(to) = self.g.kid(v, "γ") {
            let t = Self::fnd(self, to, a, psi)?;
            self.g.bind(v, t, a)?;
            t
        } else {
            return Err(anyhow!(
                "There is no way to get .{a} from {}",
                self.g.v_print(v)?
            ));
        };
        exit!(self, "#pf(ν{v}, {a}, {psi}): returning ν{}", r);
        Ok(r)
    }

    /// Dynamic dispatch.
    fn dd(&mut self, v: u32, psi: u32) -> Result<u32> {
        enter!(self, "#dd(ν{v}, {psi}): entering...");
        let psi2 = match self.g.kid(v, "ψ") {
            Some(p) => p,
            None => psi,
        };
        let r = if let Some(to) = self.g.kid(v, "ε") {
            self.dd(to, psi2)?
        } else if self.g.kid(v, "ξ").is_some() {
            self.dd(psi2, psi2)?
        } else if let Some(beta) = self.g.kid(v, "β") {
            let (a, to) = self
                .g
                .kids(beta)?
                .first()
                .ok_or(anyhow!("Can't find ν{beta}"))?
                .clone();
            let nv = self.fnd(to, a.as_str(), psi2)?;
            self.dd(nv, psi2)?
        } else if let Some(to) = self.g.kid(v, "π") {
            let nv = self.dd(to, psi2)?;
            self.apply(nv, v)?
        } else {
            v
        };
        exit!(self, "#dd(ν{v}, {psi}): returning ν{}", r);
        Ok(r)
    }

    /// Apply `v1` to `v2` and return a new vertex.
    fn apply(&mut self, v1: u32, v2: u32) -> Result<u32> {
        enter!(self, "#apply(ν{v1}, ν{v2}): entering...");
        self.depth += 1;
        let nv = self.g.next_id();
        self.g.add(nv)?;
        self.pull(nv, v1)?;
        self.push(nv, v2)?;
        exit!(
            self,
            "#apply(ν{v1}, ν{v2}): copy ν{v1}+ν{v2} created as ν{nv}"
        );
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
            if a == "π" || a == "ψ" {
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
        if a == "ρ" || a == "σ" {
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

    fn enter_it(&mut self, msg: String) -> Result<()> {
        self.depth += 1;
        self.snapshot(msg)?;
        Ok(())
    }

    fn exit_it(&mut self, msg: String) -> Result<()> {
        self.depth -= 1;
        self.snapshot(msg)?;
        Ok(())
    }

    fn snapshot(&mut self, msg: String) -> Result<()> {
        let name = fs::read_to_string("target/surge-recent.txt")?;
        let home = format!("target/surge/{name}");
        fs::create_dir_all(home.clone())?;
        let total = fs::read_dir(home.as_str())?
            .filter(|f| {
                f.as_ref()
                    .unwrap()
                    .path()
                    .as_os_str()
                    .to_str()
                    .unwrap()
                    .ends_with(".dot")
            })
            .count();
        if total == 0 {
            fs::copy("surge-make/Makefile", format!("{home}/Makefile"))?;
            fs::copy("surge-make/doc.tex", format!("{home}/doc.tex"))?;
            fs::write(format!("{home}/list.tex"), b"")?;
        }
        let pos = total + 1;
        let mut file = fs::File::create(format!("{home}/{pos}.dot"))?;
        file.write_all(self.g.to_dot().as_bytes())?;
        let mut list = OpenOptions::new()
            .write(true)
            .append(true)
            .open(format!("{home}/list.tex"))?;
        if pos == 1
            || fs::read_to_string(format!("{home}/{pos}.dot"))?
                != fs::read_to_string(format!("{home}/{}.dot", pos - 1))?
        {
            writeln!(list, "\\graph{{{pos}}}")?;
        }
        let mut log = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(format!("{home}/log.txt"))?;
        writeln!(
            log,
            "{}{}",
            "  ".repeat(self.depth),
            msg.replace("ν", "v").replace("Δ", "D")
        )?;
        let full = fs::read_to_string(format!("{home}/log.txt"))?;
        let lines = full.split("\n").collect::<Vec<&str>>();
        let max = 32;
        fs::write(
            format!("{home}/log-{pos}.txt"),
            lines
                .clone()
                .into_iter()
                .skip(cmp::max(0 as i16, lines.len() as i16 - max) as usize)
                .collect::<Vec<&str>>()
                .join("\n"),
        )?;
        Ok(())
    }
}

#[cfg(test)]
use sodg::Script;

#[cfg(test)]
use std::fs;
use std::fs::OpenOptions;

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
        let name = *path
            .split('/')
            .collect::<Vec<&str>>()
            .get(1)
            .ok_or(anyhow!("Can't understand path"))?;
        trace!("#quick_tests: {name}");
        fs::write("target/surge-recent.txt", name.as_bytes())?;
        let mut s = Script::from_str(fs::read_to_string(path.clone())?.as_str());
        let mut g = Sodg::empty();
        s.deploy_to(&mut g)?;
        trace!("Before:\n {}", g.clone().to_dot());
        let home = format!("target/surge/{}", name);
        if Path::new(home.as_str()).exists() {
            fs::remove_dir_all(home)?;
        }
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
