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
    title: String,
    subtitle: String,
}

impl Edge {
    fn new(from: u32, to: u32, title: String, subtitle: String) -> Edge {
        Edge {
            from, to, title, subtitle
        }
    }
}

pub type Lambda = fn(&mut Universe) -> u32;

struct Vertex {
    data: Data,
    lambda: Lambda
}

impl Vertex {
    pub fn empty() -> Self {
        Vertex {
            data: Data::empty(),
            lambda: |_| -> u32 { 0 }
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
        let mut uni = Universe {
            vertices: HashMap::new(),
            edges: HashMap::new(),
            next: AtomicU32::new(0)
        };
        uni.add();
        uni
    }

    // Get one vertex.
    fn vertex(&self, v: u32) -> Option<&Vertex> {
        self.vertices.get(&v)
    }

    // Find a vertex by locator.
    fn find(&mut self, _v: u32, _loc: &str) -> Result<u32, String> {
        Ok(0)
    }

    // Add a new vertex to the universe.
    pub fn add(&mut self) -> u32 {
        let v = *self.next.get_mut();
        self.vertices.insert(v, Vertex::empty());
        *self.next.get_mut() += 1;
        v
    }

    // Make an edge from v1 to v2 and puts a label on it. If the
    // label is not equal to "ρ", makes a backward edge from v2 to v1
    // and labels it as "ρ".
    pub fn bind(&mut self, v1: u32, v2: u32, a: &str) -> u32 {
        let e = self.reff(v1, v2, a, "");
        if a != "ρ" {
            self.reff(v2, v1, "ρ", "");
        }
        e
    }

    // Make an edge from v1 to v2 and puts a title and subtitle on it.
    pub fn reff(&mut self, v1: u32, v2: u32, title: &str, subtitle: &str) -> u32 {
        let e = *self.next.get_mut();
        self.edges.insert(e, Edge::new(v1, v2, title.to_string(), subtitle.to_string()));
        *self.next.get_mut() += 1;
        e
    }

    // Set atom lambda.
    pub fn atom(&mut self, v: u32, m: Lambda) {
        self.vertices.get_mut(&v).unwrap().lambda = m
    }

    // Set vertex data.
    pub fn data(&mut self, v: u32, d: Data) {
        self.vertices.get_mut(&v).unwrap().data = d;
    }

    // Dataize by locator.
    pub fn dataize(&mut self, v: u32, loc: &str) -> Result<&Data, String> {
        let id = self.find(v, loc).unwrap();
        let v = self.vertex(id).unwrap();
        Ok(&(*v).data)
    }
}

fn rand(uni: &mut Universe) -> u32 {
    let i = uni.add();
    uni.bind(0, i, "i");
    let int = uni.find(0, "𝜉.int").unwrap();
    uni.reff(i, int, "π", "");
    let d = uni.add();
    uni.bind(i, d, "Δ");
    let rnd = rand::random::<u64>();
    uni.data(d, Data::from_int(rnd));
    i
}

#[test]
fn generates_random_int() {
    let mut uni = Universe::empty();
    let v1 = uni.add();
    uni.bind(0, v1, "int");
    let v2 = uni.add();
    uni.bind(0, v2, "rand");
    uni.reff(0, v2, "x", "");
    uni.atom(v1, rand);
    assert_ne!(
        uni.dataize(0, "𝜉.x.Δ").unwrap().as_int(),
        uni.dataize(0, "𝜉.x.Δ").unwrap().as_int()
    );
}
