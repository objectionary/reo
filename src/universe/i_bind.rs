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
use anyhow::{anyhow, Result};
use log::trace;

impl Universe {
    /// Makes an edge `e1` from vertex `v1` to vertex `v2` and puts `a` label on it. If the
    /// label is not equal to `"ρ"`, makes two backward edges from `v2` to `v1`
    /// and label them as `"ρ"` an `"𝜎"`.
    pub fn bind(&mut self, e1: u32, v1: u32, v2: u32, a: &str) -> Result<()> {
        if !self.vertices.contains_key(&v1) {
            return Err(anyhow!("Can't find ν{}", v1));
        }
        if !self.vertices.contains_key(&v2) {
            return Err(anyhow!("Can't find ν{}", v2));
        }
        if self.edges.contains_key(&e1) {
            return Err(anyhow!("Edge ε{} already exists", e1));
        }
        if let Some(v) = self.edge(v1, a) {
            return Err(anyhow!(
                "Edge '{}' already exists in ν{}, arriving to ν{}",
                a,
                v1,
                v
            ));
        }
        self.edges.insert(e1, Edge::new(v1, v2, a.to_string()));
        if a != "ρ" && a != "𝜎" {
            if self.edge(v2, "ρ").is_none() {
                let e2 = self.next_id();
                self.bind(e2, v2, v1, "ρ")?;
            }
            if self.edge(v2, "𝜎").is_none() {
                let e3 = self.next_id();
                self.bind(e3, v2, v1, "𝜎")?;
            }
        }
        trace!(
            "#bind(ε{}, ν{}, ν{}, '{}'): edge added ν{}-ε{}({})>ν{}",
            e1,
            v1,
            v2,
            a,
            v1,
            e1,
            a,
            v2
        );
        Ok(())
    }
}

#[test]
fn binds_simple_vertices() -> Result<()> {
    let mut uni = Universe::empty();
    let v1 = uni.next_id();
    uni.add(v1)?;
    let v2 = uni.next_id();
    uni.add(v2)?;
    let e1 = uni.next_id();
    let k = "hello";
    uni.bind(e1, v1, v2, k)?;
    assert!(uni.inconsistencies().is_empty());
    assert_eq!(v2, uni.find(v1, k)?);
    assert_eq!(v1, uni.find(v2, "ρ")?);
    assert_eq!(v1, uni.find(v2, "𝜎")?);
    Ok(())
}

#[test]
fn pre_defined_ids() -> Result<()> {
    let mut uni = Universe::empty();
    uni.add(1)?;
    uni.add(2)?;
    let k = "a-привет";
    uni.bind(1, 1, 2, k)?;
    assert!(uni.inconsistencies().is_empty());
    assert_eq!(2, uni.find(1, k)?);
    Ok(())
}

#[test]
fn binds_two_names() -> Result<()> {
    let mut uni = Universe::empty();
    let v1 = uni.next_id();
    uni.add(v1)?;
    let v2 = uni.next_id();
    uni.add(v2)?;
    let e1 = uni.next_id();
    uni.bind(e1, v1, v2, "first")?;
    let e1 = uni.next_id();
    uni.bind(e1, v1, v2, "second")?;
    assert!(uni.inconsistencies().is_empty());
    assert_eq!(v2, uni.find(v1, "first")?);
    Ok(())
}

#[test]
fn binds_to_root() -> Result<()> {
    let mut uni = Universe::empty();
    uni.add(0)?;
    let v1 = uni.next_id();
    uni.add(v1)?;
    let e1 = uni.next_id();
    uni.bind(e1, 0, v1, "x")?;
    assert!(uni.inconsistencies().is_empty());
    assert!(uni.edge(0, "ρ").is_none());
    assert!(uni.edge(0, "𝜎").is_none());
    Ok(())
}
