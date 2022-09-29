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

use crate::universe::Universe;
use log::error;

impl Universe {
    /// Validate the Universe and return all found data
    /// inconsistencies. This is mostly used for testing.
    pub fn inconsistencies(&self) -> Vec<String> {
        let mut errors = Vec::new();
        for e in self.lost_edges() {
            errors.push(e);
        }
        for e in self.exclusive_trio() {
            errors.push(e);
        }
        for e in errors.to_vec() {
            error!("{}", e)
        }
        errors
    }

    /// Finds all edges that have lost ends.
    fn lost_edges(&self) -> Vec<String> {
        let mut errors = Vec::new();
        for (e, edge) in self.edges.iter() {
            if !self.vertices.contains_key(&edge.to) {
                errors.push(format!("Edge ε{} arrives to lost ν{}", e, edge.to));
            }
            if !self.vertices.contains_key(&edge.from) {
                errors.push(format!("Edge ε{} departs from lost ν{}", e, edge.from));
            }
        }
        errors
    }

    /// Finds all vertices that violate this rule: `π`, `φ`, or `λ` are
    /// mutually exclusive.
    fn exclusive_trio(&self) -> Vec<String> {
        let mut errors = Vec::new();
        for (v, vtx) in self.vertices.iter() {
            let attrs = self
                .edges
                .iter()
                .filter(|(_, e)| e.from == *v && (e.a == "π" || e.a == "φ"))
                .count();
            if vtx.lambda.is_some() && attrs > 0 {
                errors.push(format!("ν{} already has λ, can't have π or φ", v));
                continue;
            }
            if attrs > 1 {
                errors.push(format!("ν{} can't have both π and φ", v));
            }
        }
        errors
    }
}

#[cfg(test)]
use crate::{add, bind};

#[cfg(test)]
use anyhow::Result;

#[test]
fn finds_lost_edge() -> Result<()> {
    let mut uni = Universe::empty();
    uni.add(0)?;
    let v1 = add!(uni);
    bind!(uni, 0, v1, "foo");
    uni.vertices.remove(&v1);
    assert_eq!(3, uni.inconsistencies().len());
    Ok(())
}

#[test]
fn pi_and_phi_together() -> Result<()> {
    let mut uni = Universe::empty();
    let v1 = add!(uni);
    let v2 = add!(uni);
    bind!(uni, v1, v2, "π");
    bind!(uni, v1, v2, "φ");
    assert_eq!(1, uni.inconsistencies().len());
    Ok(())
}
