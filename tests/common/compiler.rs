// SPDX-FileCopyrightText: Copyright (c) 2022-2025 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

pub fn compile_one(sodg: &str, reo: PathBuf) -> Result<()> {
    let tmp = TempDir::new()?;
    let src = tmp.path().join("src.sodg");
    File::create(src.clone())?.write_all(sodg.as_bytes())?;
    assert_cmd::Command::cargo_bin("reo")?
        .arg("compile")
        .arg(src.as_os_str())
        .arg(reo.as_os_str())
        .assert()
        .success();
    Ok(())
}

#[test]
fn simple_compilation() -> Result<()> {
    let tmp = TempDir::new()?;
    let first = tmp.path().join("first.reo");
    compile_one(
        "
        ADD(ν0);
        ADD($ν1);
        BIND(ν0, $ν1, foo);
        ADD($ν2);
        BIND($ν1, $ν2, Δ);
        PUT($ν2, d0-bf-d1-80-d0-b8-d0-b2-d0-b5-d1-82);
        ",
        first.clone(),
    )?;
    assert!(first.exists());
    Ok(())
}
