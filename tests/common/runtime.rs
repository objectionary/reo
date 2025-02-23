// SPDX-FileCopyrightText: Copyright (c) 2022-2025 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use anyhow::{Context, Result};
use glob::glob;
use log::debug;
use sodg::Sodg;
use std::path::Path;

pub fn load_runtime() -> Result<Sodg> {
    let pack = Path::new("target/runtime.reo");
    if !pack.exists() {
        let sources = Path::new("target/eo/sodg");
        let target = Path::new("target/eo/reo");
        for f in glob(format!("{}/**/*.sodg", sources.display()).as_str())? {
            let src = f?;
            if src.is_dir() {
                continue;
            }
            let rel = src.as_path().strip_prefix(sources)?.with_extension("reo");
            let bin = target.join(rel);
            let parent = bin
                .parent()
                .context(format!("Can't get parent of {}", bin.display()))?;
            fsutils::mkdir(parent.to_str().unwrap());
            assert_cmd::Command::cargo_bin("reo")?
                .arg("compile")
                .arg(src.as_os_str())
                .arg(bin.as_os_str())
                .assert()
                .success();
            debug!("compiled {}", bin.display());
        }
        Sodg::empty().save(pack)?;
        for f in glob(format!("{}/**/*.reo", target.display()).as_str())? {
            let bin = f?;
            if bin.is_dir() {
                continue;
            }
            assert_cmd::Command::cargo_bin("reo")?
                .arg("merge")
                .arg(pack.as_os_str())
                .arg(bin.as_os_str())
                .assert()
                .success();
            debug!("merged {}", bin.display());
        }
    }
    Ok(Sodg::load(pack)?)
}

#[test]
fn loads_runtime() -> Result<()> {
    let g = load_runtime()?;
    assert!(g.len() > 0);
    Ok(())
}
