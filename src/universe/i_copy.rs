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

use crate::universe::{Edge, Universe};
use anyhow::{Context, Result};
use log::trace;

impl Universe {
    /// Deletes the edge `e1` and replaces it with a new edge `e2` coming
    /// from `v1` to a new vertex `v3`. Also, makes a new edge from `v3` to `v2`.
    pub fn copy(&mut self, e1: u32, v3: u32, e2: u32) -> Result<()> {
        let edge = self.edges.get(&e1).context(format!("Can't find ε{}", e1))?;
        let v1 = edge.from;
        let v2 = edge.to;
        let mut vtx2 = (*self
            .vertices
            .get(&v2)
            .context(format!("Can't find ν{}", v2))?)
        .clone();
        let mut vtx3 = vtx2.clone();
        let a = edge.a.clone();
        self.edges.remove(&e1);
        self.edges.insert(e2, Edge::new(v1, v3, a.to_string()));
        for e in self.edges.values_mut().filter(|e| e.from == v2) {
            e.from = v3;
        }
        let e3 = self.next_e();
        self.edges.insert(e3, Edge::new(v3, v2, "π".to_string()));
        vtx2.data = None;
        vtx2.lambda = vtx3.lambda;
        vtx2.lambda_name = vtx3.lambda_name;
        vtx3.lambda = None;
        vtx3.lambda_name = "".to_string();
        self.vertices.insert(v2, vtx2);
        self.vertices.insert(v3, vtx3);
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

#[cfg(test)]
use crate::data::Data;

#[cfg(test)]
use crate::{add, bind, copy};

#[cfg(test)]
use crate::universe::i_atom::not_implemented_yet;

#[test]
fn makes_simple_copy() -> Result<()> {
    let mut uni = Universe::empty();
    let v1 = add!(uni);
    let v2 = add!(uni);
    uni.data(v2, Data::from_int(42))?;
    let e1 = bind!(uni, v1, v2, "x");
    uni.register("niy", not_implemented_yet);
    uni.atom(v2, "niy")?;
    let v4 = add!(uni);
    bind!(uni, v2, v4, "y");
    let v3 = copy!(uni, e1);
    assert!(uni.inconsistencies().is_empty());
    assert_eq!(v2, uni.find(v1, "x.π")?);
    assert_eq!(v4, uni.find(v1, "x.y")?);
    let vtx2 = uni.vertices.get(&v2).unwrap();
    assert!(!vtx2.lambda_name.is_empty());
    assert!(vtx2.lambda.is_some());
    let vtx3 = uni.vertices.get(&v3).unwrap();
    assert!(vtx3.lambda_name.is_empty());
    assert!(vtx3.lambda.is_none());
    assert!(vtx3.data.is_some());
    println!("{}", uni.to_graph()?);
    Ok(())
}
