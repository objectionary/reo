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

use crate::org::eolang::register;
use crate::universe::Universe;
use anyhow::{Context, Result};
use log::trace;
use std::fs;
use std::path::Path;

impl Universe {
    /// Save the entire `Universe` into a binary file. The entire universe
    /// can be restored from the file.
    pub fn save(&mut self, path: &Path) -> Result<()> {
        let bytes: Vec<u8> = bincode::serialize(self).unwrap();
        let size = bytes.len();
        fs::write(path, bytes)?;
        trace!("Serialized {} bytes to {}", size, path.display());
        Ok(())
    }

    /// Load the entire `Universe` from a binary file previously
    /// created by `save()`.
    pub fn load(path: &Path) -> Result<Universe> {
        let bytes = fs::read(path)?;
        let size = bytes.len();
        let mut uni = bincode::deserialize(&bytes).unwrap();
        register(&mut uni);
        for v in Self::atoms(&uni) {
            let name = uni.vertices.get(&v).context("op")?.lambda_name.clone();
            uni.atom(v, name.as_str())?;
        }
        trace!("Deserialized {} bytes from {}", size, path.display());
        Ok(uni)
    }

    /// Get numbers of all vertices, which are atoms.
    fn atoms(uni: &Universe) -> Vec<u32> {
        uni.vertices
            .iter()
            .filter(|(_v, vtx)| !vtx.lambda_name.is_empty())
            .map(|(v, _vtx)| *v)
            .collect::<Vec<u32>>()
    }
}

#[cfg(test)]
use crate::da;

#[cfg(test)]
use tempfile::TempDir;

#[cfg(test)]
use crate::data::Data;

#[test]
fn saves_and_loads() -> Result<()> {
    let mut uni = Universe::empty();
    uni.add(0)?;
    uni.data(0, Data::from_int(0))?;
    uni.add(1)?;
    uni.bind(1, 0, 1, "foo")?;
    uni.atom(1, "S/Q")?;
    let before = da!(uni, "Q.foo.Δ").as_int()?;
    let tmp = TempDir::new()?;
    let relf = tmp.path().join("foo.relf");
    uni.save(relf.as_path())?;
    let mut second = Universe::load(relf.as_path())?;
    let after = da!(second, "Q.foo.Δ").as_int()?;
    assert_eq!(before, after);
    Ok(())
}
