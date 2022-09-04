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

use crate::universe::{Edge, Universe, Vertex};
use anyhow::{Context, Result};
use log::trace;

impl Universe {
    /// Deletes the edge `e1` and replaces it with a new edge `e2` coming
    /// from `v1` to a new vertex `v3`. Also, makes a new edge from `v3` to `v2`.
    pub fn copy(&mut self, e1: u32, v3: u32, e2: u32) -> Result<()> {
        let edge = self.edges.get(&e1).context(format!("Can't find ε{}", e1))?;
        let v1 = edge.from;
        let v2 = edge.to;
        let vtx2 = (*self
            .vertices
            .get(&v2)
            .context(format!("Can't find ν{}", v2))?)
        .clone();
        self.vertices.insert(v3, vtx2);
        let a = edge.a.clone();
        self.edges.remove(&e1);
        self.edges.insert(e2, Edge::new(v1, v3, a.to_string()));
        self.vertices.insert(v2, Vertex::empty());
        for e in self.edges.values_mut().filter(|e| e.from == v2) {
            e.from = v3;
        }
        self.vertices
            .get_mut(&v2)
            .context(format!("Can't find ν{}", v2))?
            .lambda = self
            .vertices
            .get(&v3)
            .context(format!("Can't find ν{}", v3))?
            .lambda;
        self.vertices
            .get_mut(&v3)
            .context(format!("Can't find ν{}", v3))?
            .lambda = None;
        let e3 = self.next_id();
        self.edges.insert(e3, Edge::new(v3, v2, "π".to_string()));
        trace!(
            "#copy(ε{}, ν{}, ε{}): ν{}-ε{}>ν{} restructured as ν{}-ε{}>ν{}-ε{}(π)>ν{}",
            e1,
            v3,
            e2,
            v1,
            e1,
            v2,
            v1,
            e2,
            v3,
            e3,
            v2
        );
        Ok(())
    }
}

#[test]
fn makes_simple_copy() -> Result<()> {
    let mut uni = Universe::empty();
    let v1 = uni.next_id();
    uni.add(v1)?;
    let v2 = uni.next_id();
    uni.add(v2)?;
    let e1 = uni.next_id();
    uni.bind(e1, v1, v2, "x")?;
    let v4 = uni.next_id();
    uni.add(v4)?;
    let e3 = uni.next_id();
    uni.bind(e3, v2, v4, "y")?;
    let v3 = uni.next_id();
    let e2 = uni.next_id();
    uni.copy(e1, v3, e2)?;
    assert!(uni.inconsistencies().is_empty());
    assert_eq!(v2, uni.find(v1, "x.π")?);
    assert_eq!(v4, uni.find(v1, "x.y")?);
    Ok(())
}
