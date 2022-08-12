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
use std::collections::HashMap;
use std::sync::atomic::AtomicU32;

struct Edge {
    from: u32,
    to: u32,
    a: String,
    loc: String,
}

impl Edge {
    fn new(from: u32, to: u32, a: String, loc: String) -> Edge {
        Edge {
            from, to, a, loc
        }
    }
}

pub type Error = String;

pub type Lambda = fn(&mut Universe) -> Result<u32, Error>;

struct Vertex {
    data: Data,
    lambda: Lambda
}

impl Vertex {
    pub fn empty() -> Self {
        Vertex {
            data: Data::empty(),
            lambda: |_| -> Result<u32, Error> { Ok(0) }
        }
    }
}

pub struct Universe {
    vertices: HashMap<u32, Vertex>,
    edges: HashMap<u32, Edge>,
    next: AtomicU32,
}

impl Universe {
    pub fn empty() -> Self {
        Universe {
            vertices: HashMap::new(),
            edges: HashMap::new(),
            next: AtomicU32::new(0)
        }
    }

    // Generates the next available ID for vertices and edges.
    pub fn next_id(&mut self) -> u32 {
        let i = *self.next.get_mut();
        *self.next.get_mut() += 1;
        i
    }

    // Add a new vertex to the universe.
    pub fn add(&mut self, v: u32) {
        self.vertices.insert(v, Vertex::empty());
    }

    // Makes an edge `e` from vertex `v1` to vertex `v2` and puts `a` label on it. If the
    // label is not equal to `"ρ"`, makes a backward edge from `v2` to `v1`
    // and labels it as `"ρ"`.
    pub fn bind(&mut self, e: u32, v1: u32, v2: u32, a: &str) {
        self.edges.insert(e, Edge::new(v1, v2, a.to_string(), "".to_string()));
        if a != "ρ" {
            self.edges.insert(e, Edge::new(v2, v1, "ρ".to_string(), "".to_string()));
        }
    }

    // Makes an edge `e1` from `v1` to `v2` and puts `a` title and `loc` locator on it.
    pub fn reff(&mut self, e1: u32, v1: u32, loc: &str, a: &str) {
        let v2 = self.find(v1, loc).unwrap();
        self.edges.insert(e1, Edge::new(v1, v2, a.to_string(), loc.to_string()));
    }

    // Deletes the edge `e1` and replaces it with a new edge `e2` coming
    // from `v1` to a new vertex `v3`. Also, makes a new edge from `v3` to `v2`.
    pub fn copy(&mut self, e1: u32, v3: u32, e2: u32) {
        let v1 = self.edges.get(&e1).unwrap().from;
        let v2 = self.edges.get(&e1).unwrap().to;
        let a = self.edges.get(&e1).unwrap().a.clone();
        let loc = self.edges.get(&e1).unwrap().loc.clone();
        self.edges.remove(&e1);
        self.edges.insert(e2, Edge::new(v1, v3, a.to_string(), loc.to_string()));
        let e3 = self.next_id();
        self.edges.insert(e3, Edge::new(v3, v2, "π".to_string(), "".to_string()));
    }

    // Set atom lambda.
    pub fn atom(&mut self, v: u32, m: Lambda) {
        self.vertices.get_mut(&v).unwrap().lambda = m
    }

    // Set vertex data.
    pub fn data(&mut self, v: u32, d: Data) {
        self.vertices.get_mut(&v).unwrap().data = d;
    }
}

impl Universe {
    // Get one vertex.
    fn vertex(&self, v: u32) -> Option<&Vertex> {
        self.vertices.get(&v)
    }

    // Find a vertex by locator.
    fn find(&mut self, _v: u32, _loc: &str) -> Result<u32, String> {
        Ok(0)
    }

    // Dataize by locator.
    pub fn dataize(&mut self, v: u32, loc: &str) -> Result<&Data, String> {
        let id = self.find(v, loc).unwrap();
        let v = self.vertex(id).unwrap();
        Ok(&(*v).data)
    }
}

fn rand(uni: &mut Universe) -> Result<u32, Error> {
    let e = uni.next_id();
    uni.reff(e, 0, "𝜉.int", "i");
    let i = uni.next_id();
    uni.add(i);
    let e2 = uni.next_id();
    uni.copy(e, i, e2);
    let d = uni.next_id();
    uni.add(d);
    let e3 = uni.next_id();
    uni.bind(e3, i, d, "Δ");
    let rnd = rand::random::<u64>();
    uni.data(d, Data::from_int(rnd));
    Ok(i)
}

#[test]
fn generates_random_int() {
    let mut uni = Universe::empty();
    let v1 = uni.next_id();
    uni.add(v1);
    let e1 = uni.next_id();
    uni.bind(e1, 0, v1, "int");
    let v2 = uni.next_id();
    uni.add(v2);
    let e2 = uni.next_id();
    uni.bind(e2, 0, v2, "rand");
    let e3 = uni.next_id();
    uni.reff(e3, 0, "v2", "x");
    uni.atom(v1, rand);
    assert_ne!(
        uni.dataize(0, "𝜉.x.Δ").unwrap().as_int(),
        uni.dataize(0, "𝜉.x.Δ").unwrap().as_int()
    );
}
