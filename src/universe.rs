// SPDX-FileCopyrightText: Copyright (c) 2022-2025 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use crate::{Atom, Universe};
use anyhow::{anyhow, Context, Result};
use lazy_static::lazy_static;
use log::{debug, trace};
use regex::Regex;
use sodg::Sodg;
use sodg::{Hex, Relay};
use std::cmp;
use std::collections::HashMap;
use std::fs;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
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
            snapshots: None,
        }
    }

    /// Point it to snapshots directory.
    pub fn with_snapshots(&self, p: &Path) -> Self {
        Universe {
            g: self.g.clone(),
            atoms: self.atoms.clone(),
            depth: self.depth,
            snapshots: Some(p.as_os_str().to_str().unwrap().to_string()),
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

/// I have no idea why we need to have this intermediate
/// function, but without it Relay::re doesn't compile.
fn relay_it(u: *const Universe, at: u32, a: &str) -> Result<String> {
    unsafe {
        let u1 = u as *mut Universe;
        let u2 = &mut *u1;
        Universe::mut_re(u2, at, a)
    }
}

impl Relay for Universe {
    /// Resolve a locator on a vertex, if it is not found.
    fn re(&self, at: u32, a: &str) -> Result<String> {
        relay_it(self, at, a)
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
        Err(anyhow!("Can't tie to ν{v}.{a}"))
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
        if self.depth > 0 {
            self.depth -= 1;
        }
        self.snapshot(msg)?;
        Ok(())
    }

    const COLORS: &'static str = "fillcolor=aquamarine3,style=filled,";

    /// Create a new snapshot (PDF file)
    fn snapshot(&mut self, msg: String) -> Result<()> {
        lazy_static! {
            static ref DOT_LINE: Regex = Regex::new("^ +v([0-9]+)\\[.*$").unwrap();
        }
        if self.snapshots.is_none() {
            return Ok(());
        }
        let p = self.snapshots.clone().unwrap();
        let home = Path::new(&p);
        fs::create_dir_all(home)
            .context(anyhow!("Can't create directory {}", home.to_str().unwrap()))?;
        let total = fs::read_dir(home)
            .context(anyhow!("Can't list files in {}", home.to_str().unwrap()))?
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
        debug!("{total} snapshot files already in {}", home.to_str().unwrap());
        if total == 0 {
            fs::copy("surge-make/Makefile", home.join("Makefile")).context(anyhow!(
                "Can't copy Makefile to '{}'",
                home.to_str().unwrap()
            ))?;
            fs::copy("surge-make/doc.tex", home.join("doc.tex")).context(anyhow!(
                "Can't copy doc.tex to '{}'",
                home.to_str().unwrap()
            ))?;
            fs::write(home.join("list.tex"), b"").context(anyhow!("Can't write empty list.tex"))?;
            debug!("Snapshot dir created: {}", home.to_str().unwrap());
        }
        let pos = total + 1;
        let mut before = String::new();
        if pos > 1 {
            let fname = format!("{}.dot", pos - 1);
            let b = home.join(fname.clone());
            before = fs::read_to_string(b.clone())
                .context(anyhow!(
                    "Can't read previous {fname} file from '{}'",
                    home.to_str().unwrap()
                ))?
                .replace(Self::COLORS, "");
            debug!("Previous snapshot read from: {}", Self::fprint(b));
        }
        let seen: Vec<u32> = before
            .split('\n')
            .map(|t| match &DOT_LINE.captures(t) {
                Some(m) => m.get(1).unwrap().as_str().parse().unwrap(),
                None => 0,
            })
            .collect();
        let dot = self.g.to_dot();
        let dot_file = home.join(format!("{pos}.dot"));
        fs::write(
            &dot_file,
            dot.split('\n')
                .map(|t| match &DOT_LINE.captures(t) {
                    Some(m) => {
                        let v = m.get(1).unwrap().as_str().parse::<u32>().unwrap();
                        if seen.contains(&v) {
                            t.to_string()
                        } else {
                            t.replace('[', format!("[{}", Self::COLORS).as_str())
                        }
                    }
                    None => t.to_string(),
                })
                .collect::<Vec<String>>()
                .join("\n"),
        )?;
        debug!("Dot file saved: {}", Self::fprint(dot_file.clone()));
        if dot == before {
            if pos > 0 {
                let m = Self::fprint(dot_file.clone());
                fs::remove_file(dot_file.clone()).context(anyhow!(
                    "Can't remove previous .dot file {}",
                    dot_file.to_str().unwrap()
                ))?;
                debug!("Similar dot file removed: {m}");
            }
        } else {
            let mut list = OpenOptions::new()
                .append(true)
                .open(home.join("list.tex"))
                .context(anyhow!(
                    "Can't open {}/list.tex for appending",
                    home.to_str().unwrap()
                ))?;
            writeln!(list, "\\graph{{{pos}}}")?;
        }
        let mut log = OpenOptions::new()
            .append(true)
            .create(true)
            .open(home.join("log.txt"))
            .context(anyhow!(
                "Can't open {}/log.txt for writing",
                home.to_str().unwrap()
            ))?;
        writeln!(
            log,
            "{}{}",
            "  ".repeat(self.depth),
            msg.replace('ν', "v").replace('Δ', "D")
        )?;
        let full = fs::read_to_string(home.join("log.txt"))?;
        let lines = full.split('\n').collect::<Vec<&str>>();
        let max = 32;
        fs::write(
            home.join(format!("log-{pos}.txt")),
            lines
                .clone()
                .into_iter()
                .skip(cmp::max(0i16, lines.len() as i16 - max) as usize)
                .collect::<Vec<&str>>()
                .join("\n"),
        )?;
        debug!("Log #{pos} added (lines={})", lines.len());
        Ok(())
    }

    /// Turn file name into a better visible string, for logs.
    fn fprint(f: PathBuf) -> String {
        let size = f.metadata().unwrap().len();
        format!("{} ({size} bytes)", f.to_str().unwrap())
    }
}

#[cfg(test)]
use sodg::Script;

#[cfg(test)]
use std::process::Command;

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
        trace!("\n\n#quick_tests: {name}");
        let mut s = Script::from_str(fs::read_to_string(&path)?.as_str());
        let mut g = Sodg::empty();
        s.deploy_to(&mut g)?;
        let p = format!("target/surge/{}", name);
        let home = Path::new(p.as_str());
        if home.exists() {
            fs::remove_dir_all(home).context(anyhow!(
                "Can't delete directory '{}'",
                home.to_str().unwrap()
            ))?;
        }
        let mut uni = Universe::from_graph(g).with_snapshots(home);
        uni.register("inc", inc);
        uni.register("times", times);
        let r = uni.dataize("Φ.foo");
        uni.exit_it("The end".to_string())?;
        if r.is_err() {
            assert!(Command::new("make")
                .current_dir(home)
                .spawn()
                .unwrap()
                .wait()
                .unwrap()
                .success());
        }
        let hex = r.context(anyhow!("Failure in {path}"))?;
        assert_eq!(42, hex.to_i64()?, "Failure in {path}");
    }
    Ok(())
}

#[test]
fn quick_errors() -> Result<()> {
    for path in sodg_scripts_in_dir("quick-errors") {
        trace!("#quick_errors: {path}");
        let mut s = Script::from_str(fs::read_to_string(&path)?.as_str());
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
