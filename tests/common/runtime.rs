// SPDX-FileCopyrightText: Copyright (c) 2022-2026 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use anyhow::{Context, Result};
use glob::glob;
use log::debug;
use reo::Graph;
use std::path::Path;

// @todo #281:90min Generate the EO runtime in the new SODG language. The
//  objects of EO must be compiled to 𝜑-expressions by a recent eo-maven-plugin
//  and then to .sodg files by `phino sodg`, into target/eo/sodg/. The old
//  build.rs, which ran Maven with test-pom.xml, was deleted, since the
//  eo-maven-plugin 0.35.4 printed SODG scripts in the old ADD/BIND language,
//  which REO doesn't understand anymore. Makefile and test-pom.xml still
//  use it and must be changed too. Maybe reo::EDGES will have to grow,
//  since packages, like org.eolang, have more objects than it allows.
//  When the runtime is ready, the test below must stop skipping itself
//  and deploys_and_runs_all_apps in tests/eo_test.rs must not be ignored.
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
fn loads_runtime() -> Result<()> {
    if !Path::new("target/runtime.reo").exists()
        && glob("target/eo/sodg/**/*.sodg")?.next().is_none()
    {
        eprintln!("skipping loads_runtime: no EO fixtures present");
        return Ok(());
    }
    let g = load_runtime()?;
    assert!(!g.is_empty());
    Ok(())
}
