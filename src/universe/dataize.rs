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
use crate::universe::Universe;
use anyhow::{anyhow, Context, Result};
use log::trace;
use std::collections::VecDeque;
use std::str::FromStr;

impl Universe {
    /// Dataize by absolute locator. The search always starts from the
    /// root node of the tree. It is recommended to start the locator
    /// from "Φ". If you need to find any vertex starting from non-root
    /// one, use `find` method.
    pub fn dataize(&mut self, loc: &str) -> Result<Data> {
        let id = self
            .find(0, loc)
            .context(format!("Failed to find {}", loc))?;
        let v = self
            .vertices
            .get(&id)
            .context(format!("ν{} is absent", id))?;
        let data = v
            .data
            .clone()
            .context(format!("There is no data in ν{}", id))?;
        Ok(data)
    }

    /// Find a vertex in the Universe by its locator. The search
    /// starts from the vertex `v`, but the locator may jump to
    /// the root vertex, if it starts with "Φ".
    pub fn find(&mut self, v: u32, loc: &str) -> Result<u32> {
        let mut vtx = v;
        let mut sectors = VecDeque::new();
        loc.split('.').for_each(|k| sectors.push_back(k));
        loop {
            if let Some(k) = sectors.pop_front() {
                if k.starts_with("ν") {
                    vtx = u32::from_str(&k[2..])?;
                    continue;
                }
                if k == "𝜉" {
                    vtx = vtx;
                    continue;
                }
                if k == "Φ" {
                    vtx = 0;
                    continue;
                }
                if k == "Q" {
                    vtx = 0;
                    continue;
                }
                if k == "" {
                    return Err(anyhow!("The locator is empty"));
                }
                let to = match self.edge(vtx, k) {
                    Some(v) => v,
                    None => {
                        let to = match self.edge(vtx, "φ") {
                            Some(v) => v,
                            None => match self
                                .vertices
                                .get(&vtx)
                                .context(format!("Can't find ν{}", vtx))?
                                .lambda
                                .clone()
                            {
                                Some(m) => {
                                    let to = m(self, vtx)?;
                                    trace!("#dataize({}, '{}'): atom returned {}", v, loc, to);
                                    to
                                }
                                None => {
                                    if k == "Δ" {
                                        return Ok(vtx);
                                    }
                                    return Err(anyhow!(
                                        "Can't find attribute '{}' at ν{}",
                                        k,
                                        vtx
                                    ));
                                }
                            },
                        };
                        sectors.push_front(k);
                        to
                    }
                };
                if !self.vertices.contains_key(&to) {
                    return Err(anyhow!("Can't move to ν{}.{}, ν{} is absent", vtx, k, to));
                }
                vtx = to;
            } else {
                break;
            }
        }
        Ok(vtx)
    }

    /// Find `k`-labeled edge departing from `v` and return the number
    /// of the vertex it points to.
    ///
    /// @todo #1 This method is very slow. Let's find a way how to build
    ///  some index, so that the speed of this search will be higher.
    pub fn edge(&self, v: u32, k: &str) -> Option<u32> {
        Some(self.edges.values().find(|e| e.from == v && e.a == k)?.to)
    }
}

#[test]
fn search_atom_works() -> Result<()> {
    let mut uni = Universe::empty();
    uni.add(0)?;
    let v1 = uni.next_v();
    uni.add(v1)?;
    let e1 = uni.next_e();
    uni.bind(e1, 0, v1, "a")?;
    let v2 = uni.next_v();
    uni.add(v2)?;
    let e2 = uni.next_e();
    uni.bind(e2, 0, v2, "b")?;
    let v3 = uni.next_v();
    uni.add(v3)?;
    let e4 = uni.next_e();
    uni.bind(e4, v2, v3, "c")?;
    uni.atom(v1, "S/Φ.b")?;
    assert_eq!(uni.find(v1, "Φ.a.c")?, v3);
    Ok(())
}

#[test]
fn finds_root() -> Result<()> {
    let mut uni = Universe::empty();
    uni.add(0)?;
    uni.data(0, Data::from_int(0))?;
    uni.add(1)?;
    uni.atom(1, "S/Q")?;
    assert_eq!(uni.find(1, "Δ")?, 0);
    Ok(())
}

#[test]
fn fails_if_locator_is_wrong() -> Result<()> {
    let mut uni = Universe::empty();
    uni.add(0)?;
    uni.data(0, Data::from_int(0))?;
    assert!(uni.find(0, "hello").is_err());
    Ok(())
}
