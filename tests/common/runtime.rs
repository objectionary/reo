// SPDX-FileCopyrightText: Copyright (c) 2022-2026 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use anyhow::{Context, Result};
use glob::glob;
use log::debug;
use reo::Graph;
use std::path::Path;
use std::sync::Mutex;
<<<<<<< Updated upstream
=======

/// Only one test at a time may build target/runtime.reo, while others
/// wait for it, instead of loading it half-built.
static BUILDING: Mutex<()> = Mutex::new(());
>>>>>>> Stashed changes

/// Only one test at a time may build target/runtime.reo, while others
/// wait for it, instead of loading it half-built.
static BUILDING: Mutex<()> = Mutex::new(());

// @todo #285:60min Make `phino sodg` print all objects of the EO runtime.
//  Now `make` skips 15 of 166 of them, like `directory`, `file`, and `i16`,
//  since the dealpha goal of eo-maven-plugin leaves positional arguments
//  in them, which `phino sodg` refuses to print. Fibonacci doesn't need
//  them, but other programs in eo-tests/ will. When all objects are
//  printed, the `else` branch of the .sodg rule in the Makefile, which
//  skips them, must be removed.

/// Load target/runtime.reo, building it from the .sodg files in
/// target/eo/sodg/, which `make` prints, if it is not there yet.
pub fn load_runtime() -> Result<Graph> {
    let _guard = BUILDING
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let pack = Path::new("target/runtime.reo");
    if !pack.exists() {
        build_runtime(pack)?;
    }
    Graph::load(pack)
}

/// Compile all .sodg files in target/eo/sodg/ and merge them into
/// the `pack`, through a temporary file, so that a failed build never
/// leaves a half-built pack behind.
fn build_runtime(pack: &Path) -> Result<()> {
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
    let tmp = pack.with_extension("reo.tmp");
    assert_cmd::Command::cargo_bin("reo")?
        .arg("empty")
        .arg(tmp.as_os_str())
        .assert()
        .success();
    for f in glob(format!("{}/**/*.reo", target.display()).as_str())? {
        let bin = f?;
        if bin.is_dir() {
            continue;
        }
        assert_cmd::Command::cargo_bin("reo")?
            .arg("merge")
            .arg(tmp.as_os_str())
            .arg(bin.as_os_str())
            .assert()
            .success();
        debug!("merged {}", bin.display());
    }
    std::fs::rename(&tmp, pack)?;
    Ok(())
}

#[test]
fn loads_runtime() -> Result<()> {
    let g = load_runtime()?;
    assert!(!g.is_empty());
    Ok(())
}
