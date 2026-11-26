// SPDX-FileCopyrightText: Copyright (c) 2022-2026 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use crate::graph::{next_after, rebuilt, validate, PHI, PI, RHO};
use crate::{Atom, Graph, Universe, EDGES};
use anyhow::{anyhow, bail, Context, Result};
use lazy_static::lazy_static;
use log::{debug, trace};
use regex::Regex;
use sodg::{Hex, Kind, Label};
use std::cmp;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// How deep the evaluation may go, before it is considered endless.
const MAX_DEPTH: usize = 256;

/// The capacity of the graph of an empty [`Universe`].
const MIN_CAPACITY: usize = 16;

/// The ID of the root cell, which is `Φ`.
const ROOT: usize = 0;

impl Universe {
    /// Makes an empty Universe.
    pub fn empty() -> Self {
        Self::with_graph(Graph::empty(MIN_CAPACITY), MIN_CAPACITY, 0)
    }

    /// Makes a Universe from a graph, for example, the one
    /// made by [`crate::deploy`] or loaded from a `.reo` file.
    ///
    /// # Errors
    ///
    /// If the graph uses positional attributes, like `α0`, which
    /// are not supported.
    pub fn from_graph(g: Graph) -> Result<Self> {
        validate(&g)?;
        let next = next_after(&g);
        let cap = cmp::max(MIN_CAPACITY, next * 2);
        Ok(Self::with_graph(rebuilt(&g, cap), cap, next))
    }

    fn with_graph(g: Graph, cap: usize, next: usize) -> Self {
        Universe {
            g,
            cap,
            next,
            atoms: HashMap::new(),
            depth: 0,
            snapshots: None,
        }
    }

    /// Point it to snapshots directory.
    pub fn with_snapshots(&self, p: &Path) -> Self {
        Universe {
            g: self.g.clone(),
            cap: self.cap,
            next: self.next,
            atoms: self.atoms.clone(),
            depth: self.depth,
            snapshots: Some(p.as_os_str().to_str().unwrap().to_string()),
        }
    }

    /// Registers a new atom, by the name of the λ-asset, like `L_inc`.
    pub fn register(&mut self, name: &str, a: Atom) {
        self.atoms.insert(name.to_string(), a);
        trace!("#register: atom {name} registered as {:p}", a as *const ());
    }

    /// Make a new formation with these attributes (the command `F`)
    /// and return its ID.
    ///
    /// # Errors
    ///
    /// If an attribute can't be parsed or is positional, like `α0`.
    pub fn formation(&mut self, attrs: &[&str]) -> Result<usize> {
        let labels = attrs
            .iter()
            .map(|a| Self::label(a))
            .collect::<Result<Vec<Label>>>()?;
        let c = self.fresh();
        self.g.formation(c, &labels);
        Ok(c)
    }

    /// Make a new dispatch `base.attr` (the command `D`) and return its ID.
    ///
    /// # Errors
    ///
    /// If the base is absent or the attribute can't be parsed.
    pub fn dispatch(&mut self, base: usize, attr: &str) -> Result<usize> {
        self.ensure(base)?;
        let a = Self::label(attr)?;
        let c = self.fresh();
        self.g.dispatch(c, base, a);
        Ok(c)
    }

    /// Make a new application `proto( attr ↦ arg )` (the command `A`)
    /// and return its ID.
    ///
    /// # Errors
    ///
    /// If a cell is absent or the attribute can't be parsed.
    pub fn application(&mut self, proto: usize, attr: &str, arg: usize) -> Result<usize> {
        self.ensure(proto)?;
        self.ensure(arg)?;
        let a = Self::label(attr)?;
        let c = self.fresh();
        self.g.application(c, proto, a, arg);
        Ok(c)
    }

    /// Put the cell `kid` to the attribute `attr` of the cell `c`
    /// (the command `P`).
    ///
    /// # Errors
    ///
    /// If a cell is absent, the attribute can't be parsed, or the cell
    /// has no room for one more edge.
    pub fn put(&mut self, c: usize, attr: &str, kid: usize) -> Result<()> {
        self.ensure(c)?;
        self.ensure(kid)?;
        let a = Self::label(attr)?;
        self.link(c, a, kid)
    }

    /// Set the Δ-asset of the cell `c` (the command `Δ`).
    ///
    /// # Errors
    ///
    /// If the cell is absent.
    pub fn delta(&mut self, c: usize, d: &Hex) -> Result<()> {
        self.ensure(c)?;
        self.g.delta(c, d);
        Ok(())
    }

    /// Set the λ-asset of the cell `c` (the command `λ`).
    ///
    /// # Errors
    ///
    /// If the cell is absent.
    pub fn lambda(&mut self, c: usize, name: &str) -> Result<()> {
        self.ensure(c)?;
        self.g.lambda(c, name);
        Ok(())
    }

    /// Get the Δ-asset of the cell `c`, if it is there.
    ///
    /// # Errors
    ///
    /// If the cell is absent.
    pub fn data(&self, c: usize) -> Result<Option<Hex>> {
        self.ensure(c)?;
        Ok(self.g.data(c))
    }

    /// Find the object by locator and return the name of its first
    /// void attribute, other than `ρ`, which is where a positional
    /// argument lands, like in EO's `fallback "message"`.
    ///
    /// # Errors
    ///
    /// If the object can't be found or has no void attributes.
    pub fn first_void(&mut self, loc: &str) -> Result<String> {
        let c = self.find(loc)?;
        self.g
            .attrs(c)
            .iter()
            .find(|a| **a != RHO && self.g.kid(c, **a).is_none())
            .map(ToString::to_string)
            .with_context(|| format!("There are no void attributes in {loc}"))
    }

    /// Dataize by locator, like `Φ.foo.bar` or `ν42.x`.
    ///
    /// The object is found by [`Universe::find`], and then its data
    /// is taken: from its Δ-asset, from its atom, or from its `φ`.
    ///
    /// # Errors
    ///
    /// If the object can't be found or has no data.
    pub fn dataize(&mut self, loc: &str) -> Result<Hex> {
        let c = self.find(loc)?;
        let data = self
            .dataized(c)
            .with_context(|| format!("Can't dataize {loc}"))?;
        trace!(
            "#dataize: data found for {loc} at c{c} ({} bytes): {data}",
            data.len()
        );
        Ok(data)
    }

    /// Find an object by locator and return the ID of its formation.
    ///
    /// The locator is a dot-separated list of attributes, like `Φ.foo.bar`.
    /// The search starts from the root `Φ`, unless the locator has
    /// a segment like `ν42`, which jumps to the cell no.42.
    ///
    /// # Errors
    ///
    /// If the object can't be found.
    pub fn find(&mut self, loc: &str) -> Result<usize> {
        if self.g.kind(ROOT) == Kind::Void {
            bail!("The Universe is empty, can't find {loc}");
        }
        let mut cur = ROOT;
        for seg in loc.split('.').filter(|s| !s.is_empty()) {
            if seg == "Φ" {
                cur = ROOT;
            } else if let Some(tail) = seg.strip_prefix('ν') {
                let c = usize::from_str(tail)
                    .with_context(|| format!("Can't parse '{seg}' in '{loc}'"))?;
                self.ensure(c)?;
                cur = c;
            } else {
                let a = Self::label(seg)?;
                let o = self.eval(cur)?;
                cur = self
                    .take(o, a)
                    .with_context(|| format!("Can't find {loc}"))?;
            }
        }
        self.eval(cur)
    }

    /// Get a slice of the graph, starting from the cell `c`,
    /// without `ρ` edges.
    ///
    /// # Errors
    ///
    /// If the cell is absent.
    pub fn slice(&self, c: usize) -> Result<Graph> {
        self.ensure(c)?;
        self.g.slice_some(c, |_, _, a| a != RHO)
    }

    /// Dump the graph to a file.
    ///
    /// # Errors
    ///
    /// If the file can't be written.
    pub fn dump(&self, p: &Path) -> Result<usize> {
        self.g.save(p)
    }
}

impl Universe {
    /// Evaluate the cell `c` to a formation and return its ID.
    fn eval(&mut self, c: usize) -> Result<usize> {
        self.deeper(format!("#eval(c{c})"), |u| match u.g.kind(c) {
            Kind::Formation => Ok(c),
            Kind::Dispatch(a) => {
                let base = u.eval(u.edge(c, PI)?)?;
                u.take(base, a)
            }
            Kind::Application(a) => u.apply(c, a),
            Kind::Termination => Err(anyhow!("c{c} is ⊥, the termination is reached")),
            Kind::Void => Err(anyhow!("c{c} is not an object, it can't be evaluated")),
        })
    }

    /// Evaluate the application in the cell `c`, which binds the
    /// attribute `a`, and return the ID of a new formation.
    fn apply(&mut self, c: usize, a: Label) -> Result<usize> {
        let proto = self.eval(self.edge(c, PI)?)?;
        let arg = self.edge(c, a)?;
        if !self.g.attrs(proto).contains(&a) {
            bail!("Can't apply c{proto} in c{c}, it has no attribute {a}");
        }
        if let Some(k) = self.g.kid(proto, a) {
            bail!("Can't apply c{proto} in c{c}, its attribute {a} is not void, it is c{k}");
        }
        let copy = self.copy(proto)?;
        self.link(copy, a, arg)?;
        trace!("#apply: c{c} is c{proto} copied to c{copy} with {a} ↦ c{arg}");
        Ok(copy)
    }

    /// Take the attribute `a` from the formation `o` and return the ID
    /// of the formation it is.
    fn take(&mut self, o: usize, a: Label) -> Result<usize> {
        self.deeper(format!("#take(c{o}, {a})"), |u| {
            let found = if let Some(k) = u.g.kid(o, a) {
                u.eval(k)?
            } else if u.g.attrs(o).contains(&a) {
                bail!("The attribute {a} of c{o} is void")
            } else if u.g.atom(o).is_some() {
                let r = u.call(o)?;
                let r = u.eval(r)?;
                u.take(r, a)?
            } else if a != PHI && (u.g.kid(o, PHI).is_some() || u.g.attrs(o).contains(&PHI)) {
                let phi = u.take(o, PHI)?;
                u.take(phi, a)?
            } else {
                bail!("There is no attribute {a} in c{o}")
            };
            let r = if a != RHO && u.misses_rho(found) {
                let copy = u.copy(found)?;
                u.link(copy, RHO, o)?;
                trace!("#take: c{found} copied to c{copy} with ρ ↦ c{o}");
                copy
            } else {
                found
            };
            if u.is_closed(o) {
                u.memo(o, a, r);
            }
            Ok(r)
        })
    }

    /// Dataize the cell `c`: evaluate it and take its Δ-asset, or call
    /// its atom, or dataize its `φ`.
    fn dataized(&mut self, c: usize) -> Result<Hex> {
        self.deeper(format!("#dataized(c{c})"), |u| {
            let o = u.eval(c)?;
            if let Some(d) = u.g.data(o) {
                return Ok(d);
            }
            if u.g.atom(o).is_some() {
                let r = u.call(o)?;
                return u.dataized(r);
            }
            if u.g.kid(o, PHI).is_some() || u.g.attrs(o).contains(&PHI) {
                let phi = u.take(o, PHI)?;
                return u.dataized(phi);
            }
            Err(anyhow!("There is no data in c{o}: no Δ, no λ, and no φ"))
        })
    }

    /// Call the atom of the formation `o` and return the cell it returns.
    fn call(&mut self, o: usize) -> Result<usize> {
        let name = self
            .g
            .atom(o)
            .with_context(|| format!("There is no λ-asset in c{o}"))?
            .to_string();
        let atom = *self.atoms.get(name.as_str()).with_context(|| {
            format!(
                "Can't find the atom {name} of c{o} among {} others",
                self.atoms.len()
            )
        })?;
        trace!("#call: calling {name} for c{o}...");
        let r = atom(self, o).with_context(|| format!("The atom {name} of c{o} failed"))?;
        self.ensure(r)
            .with_context(|| format!("The atom {name} of c{o} returned a wrong cell"))?;
        trace!("#call: {name} for c{o} returned c{r}");
        Ok(r)
    }

    /// Make a copy of the abstract formation `f` and return its ID.
    ///
    /// Every cell reachable from `f` through its kids is copied too,
    /// except the root, closed formations, and the cells behind `ρ` edges
    /// of formations. Edges to the copied cells point to their copies,
    /// while other edges stay as they are.
    fn copy(&mut self, f: usize) -> Result<usize> {
        let mut order = vec![];
        let mut seen = HashSet::new();
        let mut todo = vec![f];
        while let Some(c) = todo.pop() {
            if !seen.insert(c) {
                continue;
            }
            order.push(c);
            let formation = self.g.kind(c) == Kind::Formation;
            for (a, k) in self.g.kids(c) {
                if formation && *a == RHO {
                    continue;
                }
                if *k == ROOT || seen.contains(k) {
                    continue;
                }
                match self.g.kind(*k) {
                    Kind::Void => {}
                    Kind::Formation if self.is_closed(*k) => {}
                    _ => todo.push(*k),
                }
            }
        }
        let mut map = HashMap::new();
        for c in &order {
            let copy = self.fresh();
            map.insert(*c, copy);
        }
        let to = |c: usize| *map.get(&c).unwrap_or(&c);
        for c in order {
            let n = to(c);
            match self.g.kind(c) {
                Kind::Formation => {
                    let attrs = self.g.attrs(c).to_vec();
                    self.g.formation(n, &attrs);
                }
                Kind::Dispatch(a) => {
                    let base = to(self.edge(c, PI)?);
                    self.g.dispatch(n, base, a);
                }
                Kind::Application(a) => {
                    let proto = to(self.edge(c, PI)?);
                    let arg = to(self.edge(c, a)?);
                    self.g.application(n, proto, a, arg);
                }
                Kind::Termination => self.g.termination(n),
                Kind::Void => bail!("Can't copy c{c}, it is not an object"),
            }
            if let Some(d) = self.g.data(c) {
                self.g.delta(n, &d);
            }
            if let Some(name) = self.g.atom(c).map(ToString::to_string) {
                self.g.lambda(n, name.as_str());
            }
            let edges: Vec<(Label, usize)> = self.g.kids(c).map(|(a, k)| (*a, to(*k))).collect();
            for (a, k) in edges {
                self.g.put(n, a, k);
            }
        }
        let r = to(f);
        trace!("#copy: c{f} copied to c{r}, with {} cell(s)", map.len());
        Ok(r)
    }

    /// Remember that the attribute `a` of the closed formation `o` is
    /// the formation `r`, if there is room for it.
    fn memo(&mut self, o: usize, a: Label, r: usize) {
        match self.g.kid(o, a) {
            Some(k) if k == r => {}
            Some(_) => self.g.put(o, a, r),
            None if self.g.kids(o).count() < EDGES => self.g.put(o, a, r),
            None => trace!("#memo: no room in c{o} to remember {a} ↦ c{r}"),
        }
    }

    /// The formation `c` has a void attribute.
    fn is_closed(&self, c: usize) -> bool {
        self.g.kind(c) == Kind::Formation
            && self.g.attrs(c).iter().all(|a| self.g.kid(c, *a).is_some())
    }

    /// The formation `c` declares `ρ` but has no kid there.
    fn misses_rho(&self, c: usize) -> bool {
        self.g.kind(c) == Kind::Formation
            && self.g.attrs(c).contains(&RHO)
            && self.g.kid(c, RHO).is_none()
    }

    /// Put an edge, making sure the cell has room for it.
    fn link(&mut self, c: usize, a: Label, kid: usize) -> Result<()> {
        if self.g.kid(c, a).is_none() && self.g.kids(c).count() >= EDGES {
            bail!("The cell c{c} can't have more than {EDGES} edges, {a} doesn't fit");
        }
        self.g.put(c, a, kid);
        Ok(())
    }

    /// Get the kid of `c` at the edge `a`, which must be there.
    fn edge(&self, c: usize, a: Label) -> Result<usize> {
        self.g
            .kid(c, a)
            .with_context(|| format!("There is no edge {a} in c{c}"))
    }

    /// Make sure the cell `c` exists.
    fn ensure(&self, c: usize) -> Result<()> {
        if c >= self.cap || self.g.kind(c) == Kind::Void {
            bail!("The cell c{c} is absent");
        }
        Ok(())
    }

    /// Get the ID for a new cell, growing the graph if it is full.
    fn fresh(&mut self) -> usize {
        if self.next >= self.cap {
            self.cap *= 2;
            self.g = rebuilt(&self.g, self.cap);
            debug!("#fresh: the graph grew to {} cells", self.cap);
        }
        let c = self.next;
        self.next += 1;
        c
    }

    /// Parse the label of an attribute.
    fn label(s: &str) -> Result<Label> {
        let a = Label::from_str(s).with_context(|| format!("Can't parse attribute '{s}'"))?;
        if let Label::Alpha(_) = a {
            bail!("The positional attribute {a} is not supported, use a named one");
        }
        Ok(a)
    }

    /// Run the function one level deeper, making sure the recursion
    /// doesn't go too deep.
    fn deeper<T>(&mut self, msg: String, f: impl FnOnce(&mut Self) -> Result<T>) -> Result<T> {
        if self.depth >= MAX_DEPTH {
            bail!("The recursion is too deep ({} levels) at {msg}", self.depth);
        }
        self.snapshot(&msg)?;
        self.depth += 1;
        let r = f(self);
        self.depth -= 1;
        self.snapshot(&format!("{msg}: done"))?;
        r
    }

    const COLORS: &'static str = "fillcolor=aquamarine3,style=filled,";

    /// Create a new snapshot (PDF file)
    fn snapshot(&mut self, msg: &str) -> Result<()> {
        lazy_static! {
            static ref DOT_LINE: Regex = Regex::new("^ +c([0-9]+)\\[.*$").unwrap();
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
        debug!(
            "{total} snapshot files already in {}",
            home.to_str().unwrap()
        );
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
        let seen: Vec<usize> = before
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
                        let v = m.get(1).unwrap().as_str().parse::<usize>().unwrap();
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
                .skip(lines.len().saturating_sub(max))
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
use crate::deploy;

#[cfg(test)]
use std::process::Command;

#[cfg(test)]
use glob::glob;

#[cfg(test)]
use std::sync::atomic::{AtomicI64, Ordering};

#[cfg(test)]
static COUNTER: AtomicI64 = AtomicI64::new(0);

#[cfg(test)]
fn next(uni: &mut Universe, _: usize) -> Result<usize> {
    let c = uni.formation(&[])?;
    uni.delta(c, &Hex::from(COUNTER.fetch_add(1, Ordering::SeqCst)))?;
    Ok(c)
}

#[cfg(test)]
fn inc(uni: &mut Universe, c: usize) -> Result<usize> {
    let rho = uni.dataize(format!("ν{c}.ρ").as_str())?.to_i64()?;
    let r = uni.formation(&[])?;
    uni.delta(r, &Hex::from(rho + 1))?;
    Ok(r)
}

#[cfg(test)]
fn times(uni: &mut Universe, c: usize) -> Result<usize> {
    let rho = uni.dataize(format!("ν{c}.ρ").as_str())?.to_i64()?;
    let x = uni.dataize(format!("ν{c}.x").as_str())?.to_i64()?;
    let r = uni.formation(&[])?;
    uni.delta(r, &Hex::from(rho * x))?;
    Ok(r)
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

/// The kids, the Δ-asset, and the λ-asset of a cell.
#[cfg(test)]
type State = (Vec<(Label, usize)>, Option<String>, Option<String>);

#[cfg(test)]
fn state(uni: &Universe, c: usize) -> State {
    let mut kids: Vec<(Label, usize)> = uni.g.kids(c).map(|(a, k)| (*a, *k)).collect();
    kids.sort();
    (
        kids,
        uni.g.data(c).map(|d| d.print()),
        uni.g.atom(c).map(ToString::to_string),
    )
}

/// The states of all abstract formations.
#[cfg(test)]
fn abstracts(uni: &Universe) -> Vec<(usize, State)> {
    let mut all: Vec<(usize, State)> = uni
        .g
        .keys()
        .into_iter()
        .filter(|c| uni.g.kind(*c) == Kind::Formation && !uni.is_closed(*c))
        .map(|c| (c, state(uni, c)))
        .collect();
    all.sort_by_key(|(c, _)| *c);
    all
}

#[cfg(test)]
const EXAMPLE: &str = "
    F c0 int foo
    F c1 x inc φ
    F c2 ρ
    λ c2 L_inc
    P c1 inc c2
    D c3 c1 x
    P c1 φ c3
    P c0 int c1
    D c4 c0 int
    F c5
    Δ c5 00-00-00-00-00-00-00-29
    A c6 c4 x c5
    D c7 c6 inc
    P c0 foo c7
";

#[test]
fn dataizes_example_from_issue() -> Result<()> {
    let mut uni = Universe::from_graph(deploy(EXAMPLE)?)?;
    uni.register("L_inc", inc);
    let before = uni.g.len();
    assert_eq!(42, uni.dataize("Φ.foo")?.to_i64()?);
    assert_eq!(before + 5, uni.g.len(), "{:?}", uni.g);
    Ok(())
}

#[test]
fn dataizes_example_twice() -> Result<()> {
    let mut uni = Universe::from_graph(deploy(EXAMPLE)?)?;
    uni.register("L_inc", inc);
    assert_eq!(42, uni.dataize("Φ.foo")?.to_i64()?);
    let before = uni.g.len();
    assert_eq!(42, uni.dataize("Φ.foo")?.to_i64()?);
    assert_eq!(before + 1, uni.g.len(), "{:?}", uni.g);
    Ok(())
}

#[cfg(test)]
const TERMINATION: &str = "
    F c0 foo bar
    F c1 x err φ
    T c2
    P c1 err c2
    D c3 c1 x
    P c1 φ c3
    F c4
    Δ c4 00-00-00-00-00-00-00-2A
    A c5 c1 x c4
    P c0 foo c5
    T c6
    P c0 bar c6
";

#[test]
fn dataizes_around_unreached_termination() -> Result<()> {
    let mut uni = Universe::from_graph(deploy(TERMINATION)?)?;
    assert_eq!(42, uni.dataize("Φ.foo")?.to_i64()?);
    Ok(())
}

#[test]
fn fails_on_reached_termination() -> Result<()> {
    let mut uni = Universe::from_graph(deploy(TERMINATION)?)?;
    let err = uni.dataize("Φ.bar").unwrap_err();
    assert!(format!("{err:#}").contains("⊥"), "{err:#}");
    Ok(())
}

#[test]
fn dataizes_with_low_level_api() -> Result<()> {
    let mut uni = Universe::empty();
    let root = uni.formation(&["foo"])?;
    let int = uni.formation(&["x", "φ"])?;
    let phi = uni.dispatch(int, "x")?;
    uni.put(int, "φ", phi)?;
    let data = uni.formation(&[])?;
    uni.delta(data, &Hex::from(42_i64))?;
    let app = uni.application(int, "x", data)?;
    uni.put(root, "foo", app)?;
    assert_eq!(42, uni.dataize("Φ.foo")?.to_i64()?);
    Ok(())
}

#[test]
fn grows_the_graph() -> Result<()> {
    let mut uni = Universe::empty();
    let root = uni.formation(&["foo"])?;
    let mut prev = uni.formation(&[])?;
    uni.delta(prev, &Hex::from(42_i64))?;
    for _ in 0..100 {
        let c = uni.formation(&["φ"])?;
        uni.put(c, "φ", prev)?;
        prev = c;
    }
    uni.put(root, "foo", prev)?;
    assert_eq!(42, uni.dataize("Φ.foo")?.to_i64()?);
    assert!(uni.cap > 100);
    Ok(())
}

#[test]
fn rejects_positional_attributes() {
    assert!(Universe::empty().formation(&["α0"]).is_err());
}

#[test]
fn rejects_too_many_edges() -> Result<()> {
    let mut uni = Universe::empty();
    let c = uni.formation(&[])?;
    for i in 0..EDGES {
        uni.put(c, format!("a{i}").as_str(), c)?;
    }
    assert!(uni.put(c, "boom", c).is_err());
    Ok(())
}

#[test]
fn generates_different_numbers() -> Result<()> {
    let mut uni = Universe::empty();
    let root = uni.formation(&["x"])?;
    let x = uni.formation(&[])?;
    uni.lambda(x, "L_next")?;
    uni.put(root, "x", x)?;
    uni.register("L_next", next);
    let first = uni.dataize("Φ.x")?.to_i64()?;
    let second = uni.dataize("Φ.x")?.to_i64()?;
    assert_ne!(first, second);
    Ok(())
}

#[test]
fn find_absent_vertex() -> Result<()> {
    let mut uni = Universe::empty();
    uni.formation(&[])?;
    assert!(uni.dataize("Φ.foo").is_err());
    Ok(())
}

#[test]
fn fnd_absent_vertex() -> Result<()> {
    let mut uni = Universe::empty();
    uni.formation(&[])?;
    assert!(uni.dataize("ν42.foo").is_err());
    assert!(uni.dataize("ν4200000.foo").is_err());
    Ok(())
}

#[test]
fn fails_in_empty_universe() {
    assert!(Universe::empty().dataize("Φ.foo").is_err());
}

#[test]
fn fails_on_phi_cycle() -> Result<()> {
    let g = deploy(
        "
        F c0 foo
        F c1 φ
        F c2 φ
        P c1 φ c2
        P c2 φ c1
        P c0 foo c1
        ",
    )?;
    let mut uni = Universe::from_graph(g)?;
    let err = uni.dataize("Φ.foo").unwrap_err();
    assert!(
        err.chain()
            .any(|e| e.to_string().contains("The recursion is too deep")),
        "Unexpected error: {err:?}"
    );
    assert_eq!(0, uni.depth);
    Ok(())
}

#[test]
fn keeps_only_last_lines_of_large_log() -> Result<()> {
    let tmp = tempfile::TempDir::new()?;
    fs::write(tmp.path().join("log.txt"), "x\n".repeat(40000))?;
    let mut uni = Universe::empty().with_snapshots(tmp.path());
    uni.snapshot("The end")?;
    let log = fs::read_to_string(tmp.path().join("log-1.txt"))?;
    assert_eq!(32, log.split('\n').count());
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
        let g = deploy(fs::read_to_string(&path)?.as_str())?;
        let p = format!("target/surge/{}", name);
        let home = Path::new(p.as_str());
        if home.exists() {
            fs::remove_dir_all(home).context(anyhow!(
                "Can't delete directory '{}'",
                home.to_str().unwrap()
            ))?;
        }
        let mut uni = Universe::from_graph(g)?.with_snapshots(home);
        uni.register("L_inc", inc);
        uni.register("L_times", times);
        let before = abstracts(&uni);
        let r = uni.dataize("Φ.foo");
        uni.snapshot("The end")?;
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
        for (c, s) in before {
            assert_eq!(
                s,
                state(&uni, c),
                "The abstract object c{c} was modified in {path}"
            );
        }
    }
    Ok(())
}

#[test]
fn quick_errors() -> Result<()> {
    for path in sodg_scripts_in_dir("quick-errors") {
        trace!("#quick_errors: {path}");
        let r = deploy(fs::read_to_string(&path)?.as_str())
            .and_then(Universe::from_graph)
            .and_then(|mut uni| {
                uni.register("L_inc", inc);
                uni.register("L_times", times);
                uni.dataize("Φ.foo")
            });
        assert!(
            r.is_err(),
            "A failure is expected in {path}, but it didn't happen"
        );
        trace!(
            "#quick_errors: {path} failed as expected: {:#}",
            r.unwrap_err()
        );
    }
    Ok(())
}
