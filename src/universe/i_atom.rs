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
use anyhow::{anyhow, Context, Result};
use log::trace;

/// Atom that always throws an error.
pub fn not_implemented_yet(_uni: &mut Universe, _v: u32) -> Result<u32> {
    Err(anyhow!("Not implemented yet"))
}

/// Atom that searches.
pub fn search(uni: &mut Universe, v: u32) -> Result<u32> {
    let loc = uni
        .vertices
        .get(&v)
        .context(format!("Can't find ν{}", v))?
        .search
        .clone();
    uni.find(v, loc.as_str())
}

impl Universe {
    /// Set atom lambda `m` to the vertex `v1`. The atom has to be
    /// registered in the Universe with the method `register`.
    pub fn atom(&mut self, v1: u32, m: &str) -> Result<()> {
        let vtx = self
            .vertices
            .get_mut(&v1)
            .context(format!("Can't find ν{}", v1))?;
        if m.starts_with("S/") {
            vtx.lambda = Some(search);
            vtx.search = m.chars().skip(2).collect();
            trace!(
                "#atom(ν{}, '{}'): lambda SEARCH set to '{}'",
                v1,
                m,
                vtx.search
            );
        } else {
            vtx.lambda = Some(match self.atoms.get(m) {
                Some(a) => {
                    trace!("#atom(ν{}, '{}'): lambda found and set", v1, m);
                    *a
                }
                None => {
                    trace!("#atom(ν{}, '{}'): lambda NOT found but set to NIY", v1, m);
                    not_implemented_yet
                }
            });
        }
        vtx.lambda_name = m.to_string();
        Ok(())
    }
}

#[cfg(test)]
fn dummy(_uni: &mut Universe, _v: u32) -> Result<u32> {
    Ok(0)
}

#[cfg(test)]
use crate::add;

#[test]
fn evaluates_dummy_atom() -> Result<()> {
    let mut uni = Universe::empty();
    uni.register("dummy", dummy);
    let v1 = add!(uni);
    uni.add(0)?;
    uni.atom(v1, "dummy")?;
    assert!(uni.inconsistencies().is_empty());
    assert_eq!(0, uni.find(v1, "Δ")?);
    Ok(())
}
