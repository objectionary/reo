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
mod i_add;
mod i_atom;
mod i_bind;
mod i_copy;
mod i_data;
mod serialization;

use crate::data::Data;
use anyhow::Result;
use log::error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Serialize, Deserialize)]
struct Edge {
    from: u32,
    to: u32,
    a: String,
}

impl Edge {
    fn new(from: u32, to: u32, a: String) -> Edge {
        Edge { from, to, a }
    }

    /// Make a copy of itself.
    pub fn clone(&self) -> Self {
        Edge {
            from: self.from.clone(),
            to: self.to.clone(),
            a: self.a.clone(),
        }
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

    /// Merge universe into inself
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

    /// Validate the Universe and return all found data
    /// inconsistencies. This is mostly used for testing.
    pub fn inconsistencies(&self) -> Vec<String> {
        let mut errors = Vec::new();
        for e in self.edges.iter() {
            if !self.vertices.contains_key(&e.1.to) {
                errors.push(format!("Edge ε{} arrives to lost ν{}", e.0, e.1.to));
            }
            if !self.vertices.contains_key(&e.1.from) {
                errors.push(format!("Edge ε{} departs from lost ν{}", e.0, e.1.from));
            }
        }
        for e in errors.to_vec() {
            error!("{}", e)
        }
        errors
    }
}

#[cfg(test)]
fn rand(uni: &mut Universe, _v: u32) -> Result<u32> {
    let v2 = uni.find(0, "int")?;
    let e1 = uni.next_e();
    uni.bind(e1, 0, v2, format!("i{}", e1).as_str())?;
    let v3 = uni.next_v();
    let e2 = uni.next_e();
    uni.copy(e1, v3, e2)?;
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
    let v1 = uni.next_v();
    uni.add(v1)?;
    let e1 = uni.next_e();
    uni.bind(e1, 0, v1, "int")?;
    let v2 = uni.next_v();
    uni.add(v2)?;
    let e2 = uni.next_e();
    uni.bind(e2, 0, v2, "rand")?;
    let e3 = uni.next_e();
    uni.bind(e3, 0, v2, "x")?;
    uni.register("rand", rand);
    uni.atom(v2, "rand")?;
    let first = uni.dataize("Φ.x.Δ")?.as_int()?;
    let second = uni.dataize("Φ.x.Δ")?.as_int()?;
    assert_ne!(first, second);
    Ok(())
}
