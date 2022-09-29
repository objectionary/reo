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

use crate::gmi::Gmi;
use crate::org::eolang::register;
use crate::universe::Universe;
use anyhow::{Context, Result};
use glob::glob;
use log::{info, trace};
use std::collections::HashMap;
use std::path::Path;

/// Deploy a directory of `*.gmi` files to a new `Universe`. Returns
/// total number of GMI instructions deployed to the Universe.
pub fn setup(uni: &mut Universe, dir: &Path) -> Result<u32> {
    register(uni);
    let mut pkgs: HashMap<String, u32> = HashMap::new();
    let mut total = 0;
    for f in glob(format!("{}/**/*.gmi", dir.display()).as_str())? {
        let p = f?;
        if p.is_dir() {
            continue;
        }
        let path = p.as_path();
        let rel = path.strip_prefix(dir)?;
        trace!("#setup: deploying {}...", path.display());
        let pkg = rel
            .parent()
            .context(format!("Can't get parent from '{}'", rel.display()))?
            .to_str()
            .context(format!("Can't turn path '{}' to str", rel.display()))?
            .replace("/", ".");
        let mut gmi = Gmi::from_file(path).context(format!("Can't read {}", path.display()))?;
        let mut root: u32 = 0;
        let mut pk = "".to_owned();
        trace!("#setup: package is '{}'", pkg);
        for p in pkg.split(".").filter(|i| !i.is_empty()) {
            pk.push_str(format!(".{}", p).as_str());
            match pkgs.get(&pk) {
                Some(v) => {
                    root = *v;
                }
                None => {
                    let v = uni.next_v();
                    uni.add(v)?;
                    let e = uni.next_e();
                    uni.bind(e, root, v, p)?;
                    root = v;
                    pkgs.insert(pk.clone(), root);
                }
            }
        }
        gmi.set_root(root);
        trace!("#setup: root set to ν{}", root);
        let instructions = gmi
            .deploy_to(uni)
            .context(format!("Failed to deploy '{}'", path.display()))?;
        info!(
            "Deployed {} instructions from {}",
            instructions,
            path.display()
        );
        total += instructions;
    }
    Ok(total)
}

#[cfg(test)]
use tempfile::TempDir;

#[cfg(test)]
use std::fs::File;

#[cfg(test)]
use std::io::Write;

#[test]
fn sets_up_simple_directory() -> Result<()> {
    let tmp = TempDir::new()?;
    File::create(tmp.path().join("foo.gmi"))?.write_all(
        "
        ADD('$ν1');
        BIND('$ε2', 'ν0', '$ν1', 'foo');
        DATA('$ν1', '00 00 00 00 00 00 00 01');
        "
        .as_bytes(),
    )?;
    let mut uni = Universe::empty();
    uni.add(0)?;
    setup(&mut uni, tmp.path())?;
    assert_eq!(1, uni.dataize("Φ.foo")?.as_int()?);
    Ok(())
}

#[test]
fn sets_up_with_subdirectories() -> Result<()> {
    let tmp = TempDir::new()?;
    std::fs::create_dir(tmp.path().join("abc"))?;
    File::create(tmp.path().join("abc/foo.gmi"))?.write_all(
        "
        ADD('$ν1');
        BIND('$ε2', 'ν0', '$ν1', 'foo');
        DATA('$ν1', '00 00 00 00 00 00 00 02');
        "
        .as_bytes(),
    )?;
    let mut uni = Universe::empty();
    uni.add(0)?;
    setup(&mut uni, tmp.path())?;
    assert_eq!(2, uni.dataize("Φ.abc.foo")?.as_int()?);
    Ok(())
}
