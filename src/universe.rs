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

mod dataize;
mod graph;
mod i_add;
mod i_atom;
mod i_bind;
mod i_copy;
mod i_data;
mod inspect;
mod serialization;

use crate::data::Data;
use anyhow::Result;
use log::error;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;

#[derive(Clone, Serialize, Deserialize, Eq, PartialOrd, PartialEq, Ord)]
struct Edge {
    from: u32,
    to: u32,
    a: String,
}

impl Edge {
    fn new(from: u32, to: u32, a: String) -> Edge {
        Edge { from, to, a }
    }
}

pub type Error = String;

pub type Lambda = fn(&mut Universe, v: u32) -> Result<u32>;

#[derive(Serialize, Deserialize)]
struct Vertex {
    data: Option<Data>,
    lambda_name: String,
    #[serde(skip_serializing, skip_deserializing)]
    lambda: Option<Lambda>,
    #[serde(skip_serializing, skip_deserializing)]
    search: String,
}

impl Vertex {
    pub fn empty() -> Self {
        Vertex {
            data: None,
            lambda: None,
            lambda_name: "".to_string(),
            search: "".to_string(),
        }
    }

    /// Make a copy of itself.
    pub fn clone(&self) -> Self {
        Vertex {
            data: self.data.clone(),
            lambda: self.lambda.clone(),
            lambda_name: self.lambda_name.clone(),
            search: self.search.clone(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Universe {
    vertices: HashMap<u32, Vertex>,
    edges: HashMap<u32, Edge>,
    #[serde(skip_serializing, skip_deserializing)]
    atoms: HashMap<String, Lambda>,
    #[serde(skip_serializing, skip_deserializing)]
    latest_v: u32,
    #[serde(skip_serializing, skip_deserializing)]
    latest_e: u32,
}

impl fmt::Debug for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut lines = vec![];
        for (i, v) in self.vertices.iter() {
            let mut attrs = self
                .edges
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
    /// Makes an empty Universe, with no vertices and no edges.
    pub fn empty() -> Self {
        Universe {
            vertices: HashMap::new(),
            edges: HashMap::new(),
            atoms: HashMap::new(),
            latest_v: 0,
            latest_e: 0,
        }
    }

    /// Generates the next available ID for a new edge.
    pub fn next_e(&mut self) -> u32 {
        loop {
            self.latest_e += 1;
            if !self.edges.contains_key(&self.latest_e) {
                break;
            }
        }
        self.latest_e
    }

    /// Generates the next available ID for a new vertex.
    pub fn next_v(&mut self) -> u32 {
        loop {
            self.latest_v += 1;
            if !self.vertices.contains_key(&self.latest_v) {
                break;
            }
        }
        self.latest_v
    }

    /// Registers a new atom.
    pub fn register(&mut self, name: &str, m: Lambda) {
        self.atoms.insert(name.to_string(), m);
    }

    /// Merge this new universe into itself.
    pub fn merge(&mut self, uni: &Universe) {
        let mut matcher: HashMap<u32, u32> = HashMap::new();
        for vert in uni.vertices.iter() {
            let mut id = 0;
            if *vert.0 != 0 {
                id = self.next_v();
            }
            matcher.insert(*vert.0, id);
            self.vertices.insert(id, vert.1.clone());
        }
        for edge in uni.edges.iter() {
            let id = self.next_e();
            let edge = Edge {
                from: *matcher.get(&edge.1.from).unwrap(),
                to: *matcher.get(&edge.1.to).unwrap(),
                a: edge.1.a.clone(),
            };
            self.edges.insert(id, edge);
        }
    }

    /// Take a slice of the universe, keeping only the vertex specified
    /// by the locator.
    pub fn slice(&mut self, loc: String) -> Result<Universe> {
        let mut todo = HashSet::new();
        let mut done = HashSet::new();
        todo.insert(self.find(0, loc.as_str())?);
        loop {
            if todo.is_empty() {
                break;
            }
            let before: Vec<u32> = todo.drain().collect();
            for v in before {
                done.insert(v);
                for to in self
                    .edges
                    .values()
                    .filter(|e| e.a != "ρ" && e.a != "σ")
                    .filter(|e| e.from == v)
                    .map(|e| e.to)
                {
                    if done.contains(&to) {
                        continue;
                    }
                    done.insert(to);
                    todo.insert(to);
                }
            }
        }
        let mut new_vertices = HashMap::new();
        for (v, vtx) in self.vertices.iter().filter(|(v, _)| done.contains(v)) {
            new_vertices.insert(*v, vtx.clone());
        }
        let mut new_edges = HashMap::new();
        for (e, edge) in self
            .edges
            .iter()
            .filter(|(_, edge)| done.contains(&edge.from) || done.contains(&edge.to))
        {
            new_edges.insert(*e, edge.clone());
        }
        Ok(Universe {
            vertices: new_vertices,
            edges: new_edges,
            atoms: HashMap::new(),
            latest_v: self.latest_v,
            latest_e: self.latest_e,
        })
    }

    /// Validate the Universe and return all found data
    /// inconsistencies. This is mostly used for testing.
    pub fn inconsistencies(&self) -> Vec<String> {
        let mut errors = Vec::new();
        for (e, edge) in self.edges.iter() {
            if !self.vertices.contains_key(&edge.to) {
                errors.push(format!("Edge ε{} arrives to lost ν{}", e, edge.to));
            }
            if !self.vertices.contains_key(&edge.from) {
                errors.push(format!("Edge ε{} departs from lost ν{}", e, edge.from));
            }
        }
        for e in errors.to_vec() {
            error!("{}", e)
        }
        errors
    }
}

#[cfg(test)]
use crate::{add, bind, copy};

#[cfg(test)]
fn rand(uni: &mut Universe, _v: u32) -> Result<u32> {
    let v2 = uni.find(0, "int")?;
    let e1 = bind!(uni, 0, v2, format!("i{}", v2).as_str());
    let v3 = copy!(uni, e1);
    uni.data(v3, Data::from_int(rand::random::<i64>()))?;
    Ok(v3)
}

#[test]
fn generates_unique_ids() -> Result<()> {
    let mut uni = Universe::empty();
    let first = uni.next_v();
    let second = uni.next_v();
    assert_ne!(first, second);
    Ok(())
}

#[test]
fn safely_generates_unique_ids() -> Result<()> {
    let mut uni = Universe::empty();
    uni.add(1)?;
    let v = uni.next_v();
    uni.add(v)?;
    Ok(())
}

#[test]
fn generates_random_int() -> Result<()> {
    let mut uni = Universe::empty();
    uni.add(0)?;
    let v1 = add!(uni);
    bind!(uni, 0, v1, "int");
    let v2 = add!(uni);
    bind!(uni, 0, v2, "rand");
    bind!(uni, 0, v2, "x");
    uni.register("rand", rand);
    uni.atom(v2, "rand")?;
    let first = uni.dataize("Φ.x.Δ")?.as_int()?;
    let second = uni.dataize("Φ.x.Δ")?.as_int()?;
    assert_ne!(first, second);
    Ok(())
}

#[test]
fn makes_a_slice() -> Result<()> {
    let mut uni = Universe::empty();
    uni.add(0)?;
    let v1 = add!(uni);
    bind!(uni, 0, v1, "foo");
    let v2 = add!(uni);
    bind!(uni, 0, v2, "bar");
    let slice = uni.slice("Φ.bar".to_string())?;
    assert_eq!(1, slice.vertices.len());
    Ok(())
}
