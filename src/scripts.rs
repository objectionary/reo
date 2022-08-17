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

use std::collections::HashMap;
use std::path::Path;
use crate::data::Data;
use glob::glob;
use crate::universe::Universe;
use anyhow::{Context, Result};
use crate::gmi::Gmi;

/// Deploy a directory of `*.gmi` files to a new `Universe`.
pub fn setup(dir: &Path) -> Result<Universe> {
    let mut uni = Universe::empty();
    uni.add(0);
    let mut pkgs : HashMap<String, u32> = HashMap::new();
    for f in glob(format!("{}/**/*.gmi", dir.display()).as_str())? {
        let p = f?;
        let path = p.as_path();
        let pkg = path.parent()
            .context(format!("Can't get parent from '{}'", path.display()))?
            .to_str()
            .context(format!("Can't turn path '{}' to str", path.display()))?
            .replace("/", ".");
        let mut gmi = Gmi::from_file(path)?;
        let mut root : u32 = 0;
        let mut pk = "".to_owned();
        for p in pkg.split(".") {
            pk.push_str(format!(".{}", p).as_str());
            match pkgs.get(&pk) {
                Some(v) => {
                    root = *v;
                },
                None => {
                    let v = uni.next_id();
                    uni.add(v);
                    let e = uni.next_id();
                    uni.bind(e, root, v, p);
                    root = v;
                    pkgs.insert(pk.clone(), root);
                }
            }
        }
        gmi.set_root(root);
        gmi.deploy_to(&mut uni)?;
    }
    Ok(uni)
}

/// Makes a copy of `int` in the Universe. It is assumed
/// that it already exists there.
pub fn copy_of_int(uni: &mut Universe, data: i64) -> u32 {
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
    uni.data(d, Data::from_int(data));
    i
}
