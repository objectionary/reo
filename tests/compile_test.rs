// SPDX-FileCopyrightText: Copyright (c) 2022-2026 Yegor Bugayenko
// SPDX-License-Identifier: MIT

mod common;

use anyhow::Result;
use predicates::prelude::predicate;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn compiles_int() -> Result<()> {
    let src = "target/eo/sodg/org/eolang/int.sodg";
    if !Path::new(src).exists() {
        eprintln!("skipping compiles_int: EO fixture '{src}' is absent (Maven step skipped)");
        return Ok(());
    }
    let tmp = TempDir::new()?;
    let bin = tmp.path().join("int.reo");
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .arg("--verbose")
        .arg("compile")
        .arg(src)
        .arg(bin.as_os_str())
        .assert()
        .success();
    assert!(bin.exists());
    Ok(())
}

#[test]
fn skips_compilation_if_file_present() -> Result<()> {
    let src = "target/eo/sodg/org/eolang/tuple.sodg";
    if !Path::new(src).exists() {
        eprintln!(
            "skipping skips_compilation_if_file_present: EO fixture '{src}' is absent \
             (Maven step skipped)"
        );
        return Ok(());
    }
    let tmp = TempDir::new()?;
    let bin = tmp.path().join("float.reo");
    for _ in 0..2 {
        assert_cmd::Command::cargo_bin("reo")
            .unwrap()
            .arg("--verbose")
            .arg("compile")
            .arg(src)
            .arg(bin.as_os_str())
            .assert()
            .success();
    }
    Ok(())
}

#[test]
fn fails_when_file_is_absent() -> Result<()> {
    let path = "/usr/boom-is-absent";
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .arg("compile")
        .arg(path)
        .arg(path)
        .assert()
        .code(1)
        .stderr(predicate::str::contains("not found"));
    Ok(())
}
