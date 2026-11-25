// SPDX-FileCopyrightText: Copyright (c) 2022-2026 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use anyhow::{Context, Result};
use glob::glob;
use log::debug;
use reo::Graph;
use std::path::Path;

// @todo #285:90min Make REO deploy the .sodg files that `make` prints into
//  target/eo/sodg/ with the help of eo-maven-plugin and `phino sodg`.
//  Now REO compiles only a few of them, fibonacci among them. The unit
//  tests of the EO runtime are printed as `p🌵...` and `n🌵...` attributes,
//  which are often longer than 32 bytes and give formations more than
//  reo::EDGES edges, so they must be dropped; phino prints `⊥` with the
//  `T` command, which sodg doesn't know yet (objectionary/sodg.rs#327);
//  the dealpha goal leaves a few positional attributes in the runtime,
//  like in `directory` and `file`, and `phino sodg` refuses to print
//  them; and the atoms, like `L_number_plus`, must be implemented for
//  64-bit floats. When the runtime is ready, the test below and
//  deploys_and_runs_all_apps in tests/eo_test.rs must not be ignored,
//  and the cargo workflow must install Maven and phino and run `make`
//  before `cargo test`.
pub fn load_runtime() -> Result<Graph> {
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
        assert_cmd::Command::cargo_bin("reo")?
            .arg("empty")
            .arg(pack.as_os_str())
            .assert()
            .success();
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
    Graph::load(pack)
}

#[test]
#[ignore = "REO can't deploy the .sodg files of the EO runtime yet, see #285"]
fn loads_runtime() -> Result<()> {
    let g = load_runtime()?;
    assert!(!g.is_empty());
    Ok(())
}
