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

use crate::data::Data;
use std::collections::VecDeque;
use std::collections::HashMap;
use std::fmt;
use anyhow::{anyhow, Context, Result};
use std::str::FromStr;
use log::trace;
use crate::scripts::copy_of_int;

struct Edge {
    from: u32,
    to: u32,
    a: String,
    k: String,
}

impl Edge {
    fn new(from: u32, to: u32, a: String, k: String) -> Edge {
        Edge {
            from, to, a, k
        }
    }
}

pub type Error = String;

pub type Lambda = fn(&mut Universe, v: u32) -> Result<u32>;

struct Vertex {
    data: Option<Data>,
    lambda: Option<String>
}

impl Vertex {
    pub fn empty() -> Self {
        Vertex {
            data: None,
            lambda: None
        }
    }
}

pub struct Universe {
    vertices: HashMap<u32, Vertex>,
    edges: HashMap<u32, Edge>,
    atoms: HashMap<String, Lambda>
}

impl fmt::Debug for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut lines = vec![];
        for (i, v) in self.vertices.iter() {
            let mut attrs = self.edges
                .iter()
                .filter(|(_, e)| e.from == *i)
                .map(|(j, e)| format!("\n\t{} ε{}➞ ν{}", e.a, j, e.to))
                .collect::<Vec<String>>();
            if let Some(d) = &v.data {
                attrs.push(format!("{}", d.as_hex()));
            }
            if let Some(_) = &v.lambda {
                attrs.push("λ".to_string());
            }
            lines.push(format!("ν{} -> ⟦{}⟧", i, attrs.join(", ")));
        }
        f.write_str(lines.join("\n").as_str())
    }
}

impl Universe {
    pub fn empty() -> Self {
        Universe {
            vertices: HashMap::new(),
            edges: HashMap::new(),
            atoms: HashMap::new()
        }
    }

    /// Generates the next available ID for vertices and edges.
    pub fn next_id(&mut self) -> u32 {
        let max = self.vertices.keys().max();
        let mut i = 0;
        if let Some(m) = max {
            i = i.max(*m);
        }
        if let Some(k) = self.edges.keys().max() {
            i = i.max(*k)
        }
        trace!("#next_id() -> {}", i);
        i + 1
    }

    /// Registers a new atom.
    pub fn register(&mut self, name: &str, m: Lambda) {
        self.atoms.insert(name.to_string(), m);
    }
}

impl Universe {
    // Add a new vertex to the universe.
    pub fn add(&mut self, v: u32) {
        self.vertices.insert(v, Vertex::empty());
        trace!("#add({}): new vertex added", v);
    }

    // Makes an edge `e` from vertex `v1` to vertex `v2` and puts `a` label on it. If the
    // label is not equal to `"ρ"`, makes a backward edge from `v2` to `v1`
    // and labels it as `"ρ"`.
    pub fn bind(&mut self, e: u32, v1: u32, v2: u32, a: &str) {
        self.edges.insert(e, Edge::new(v1, v2, a.to_string(), "".to_string()));
        trace!("#bind({}, {}, {}, '{}'): edge added", e, v1, v2, a);
        if a != "ρ" {
            let e1 = self.next_id();
            self.edges.insert(e1, Edge::new(v2, v1, "ρ".to_string(), "".to_string()));
            trace!("#bind({}, {}, {}, '{}'): backward ρ-edge added", e, v1, v2, a);
        }
    }

    // Makes an edge `e1` from `v1` to `v2` and puts `a` title and `k` locator on it.
    pub fn reff(&mut self, e1: u32, v1: u32, k: &str, a: &str) {
        let v2 = self.find(v1, k).unwrap();
        self.edges.insert(e1, Edge::new(v1, v2, a.to_string(), k.to_string()));
        trace!("#reff({}, {}, '{}', '{}'): edge added", e1, v1, k, a);
    }

    // Deletes the edge `e1` and replaces it with a new edge `e2` coming
    // from `v1` to a new vertex `v3`. Also, makes a new edge from `v3` to `v2`.
    pub fn copy(&mut self, e1: u32, v3: u32, e2: u32) {
        let v1 = self.edges.get(&e1).unwrap().from;
        let v2 = self.edges.get(&e1).unwrap().to;
        let a = self.edges.get(&e1).unwrap().a.clone();
        let k = self.edges.get(&e1).unwrap().k.clone();
        self.edges.remove(&e1);
        trace!("#copy({}, {}, {}): edge {} removed", e1, v3, e2, e1);
        self.edges.insert(e2, Edge::new(v1, v3, a.to_string(), k.to_string()));
        trace!("#copy({}, {}, {}): edge {} added", e1, v3, e2, e2);
        let e3 = self.next_id();
        self.edges.insert(e3, Edge::new(v3, v2, "π".to_string(), "".to_string()));
        trace!("#copy({}, {}, {}): π-edge {} added", e1, v3, e2, e3);
    }

    // Set atom lambda.
    pub fn atom(&mut self, v: u32, m: &str) {
        self.vertices.get_mut(&v).unwrap().lambda = Some(m.to_string());
        trace!("#atom({}, ...): lambda set", v);
    }

    // Set vertex data.
    pub fn data(&mut self, v: u32, d: Data) {
        self.vertices.get_mut(&v).unwrap().data = Some(d.clone());
        trace!("#data({}, '{}'): data set", v, d.as_hex());
    }

    /// Dataize by absolute locator.
    pub fn dataize(&mut self, loc: &str) -> Result<Data> {
        let id = self.find(0, loc)?;
        let v = self.vertex(id).context(format!("ν{} is absent", id))?;
        let data = v.data.clone().context(format!("There is no data in ν{}", id))?;
        Ok(data)
    }
}

impl Universe {
    // Get one vertex.
    fn vertex(&self, v: u32) -> Option<&Vertex> {
        self.vertices.get(&v)
    }

    // Find a vertex by locator.
    fn find(&mut self, v: u32, loc: &str) -> Result<u32> {
        let mut vtx = v;
        let mut sectors = VecDeque::new();
        loc.split(".").for_each(|k| sectors.push_back(k));
        loop {
            if let Some(k) = sectors.pop_front() {
                if k.starts_with("ν") {
                    vtx = u32::from_str(&k[2..])?
                } else if k == "𝜉" {
                    vtx = vtx;
                } else if k == "Φ" {
                    vtx = 0;
                } else {
                    let to = match self.edges.values().find(|e| e.from == vtx && e.a == k) {
                        Some(e) => e.to,
                        None => {
                            let to = match self.edges.values().find(|e| e.from == vtx && e.a == "φ") {
                                Some(e) => e.to,
                                None => match self.vertices.get(&vtx).context(format!("Can't find ν{}", vtx))?.lambda.clone() {
                                    Some(m) => self.atoms.get(m.as_str()).context(format!("No atom '{}'", m))?(self, vtx)?,
                                    None => {
                                        return Err(anyhow!("Can't continue as ν{}.{}", vtx, k));
                                    }
                                }
                            };
                            sectors.push_front(k);
                            to
                        }
                    };
                    if !self.vertices.contains_key(&to) {
                        return Err(anyhow!("Can't move to ν{}.{}, ν{} is absent", vtx, k, to));
                    }
                    vtx = to;
                }
            } else {
                break;
            }
        }
        Ok(vtx)
    }
}

#[cfg(test)]
fn rand(uni: &mut Universe, _v: u32) -> Result<u32> {
    let rnd = rand::random::<i64>();
    let i = copy_of_int(uni, rnd);
    Ok(i)
}

#[test]
fn generates_unique_ids() {
    let mut uni = Universe::empty();
    let first = uni.next_id();
    assert_eq!(first, uni.next_id());
    uni.add(first);
    assert_ne!(first, uni.next_id());
}

#[test]
fn generates_random_int() {
    let mut uni = Universe::empty();
    uni.add(0);
    let v1 = uni.next_id();
    uni.add(v1);
    let e1 = uni.next_id();
    uni.bind(e1, 0, v1, "int");
    let v2 = uni.next_id();
    uni.add(v2);
    let e2 = uni.next_id();
    uni.bind(e2, 0, v2, "rand");
    let e3 = uni.next_id();
    uni.reff(e3, 0, "Φ.rand", "x");
    uni.register("rand", rand);
    uni.atom(v2, "rand");
    println!("{uni:?}");
    assert_ne!(
        uni.dataize("Φ.x.Δ").unwrap().as_int().unwrap(),
        uni.dataize("Φ.x.Δ").unwrap().as_int().unwrap()
    );
}
