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

use crate::universe::{Universe, Vertex};
use anyhow::{anyhow, Result};
use log::trace;

impl Universe {
    /// Add a new vertex `v1` to the universe.
    pub fn add(&mut self, v1: u32) -> Result<()> {
        if self.vertices.contains_key(&v1) {
            return Err(anyhow!("Vertex ν{} already exists", v1));
        }
        self.vertices.insert(v1, Vertex::empty());
        trace!("#add(ν{}): new vertex added", v1);
        Ok(())
    }
}

#[test]
fn adds_simple_vertex() -> Result<()> {
    let mut uni = Universe::empty();
    let v1 = uni.next_v();
    uni.add(v1)?;
    assert!(uni.inconsistencies().is_empty());
    assert_eq!(v1, uni.find(v1, "ξ")?);
    Ok(())
}
