// SPDX-FileCopyrightText: Copyright (c) 2022-2026 Yegor Bugayenko
// SPDX-License-Identifier: MIT

mod common;

use crate::common::compiler::compile_one;
use anyhow::Result;
use predicates::prelude::predicate;
use std::fs;
use tempfile::TempDir;

#[test]
fn prints_dot() -> Result<()> {
    let tmp = TempDir::new()?;
    let bin = tmp.path().join("first.reo");
    let dot = tmp.path().join("first.dot");
    compile_one(
        "
        F c0 foo
        F c1
        Δ c1 d0-bf-d1-80-d0-b8-d0-b2-d0-b5-d1-82
        P c0 foo c1
        ",
        bin.clone(),
    )?;
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .current_dir(tmp.path())
        .arg("dot")
        .arg(bin.as_os_str())
        .arg(dot.as_os_str())
        .assert()
        .success();
    assert!(fs::read_to_string(dot)?.contains("c0 -> c1 [label=\"foo\"]"));
    Ok(())
}

#[test]
fn rejects_absent_root() -> Result<()> {
    let tmp = TempDir::new()?;
    let bin = tmp.path().join("first.reo");
    compile_one("F c0", bin.clone())?;
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .current_dir(tmp.path())
        .arg("dot")
        .arg("--root=7")
        .arg(bin.as_os_str())
        .assert()
        .failure()
        .stderr(predicate::str::contains("There is no cell c7"));
    Ok(())
}

#[test]
fn mentions_stdout_fallback_in_help() {
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .arg("dot")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("prints to stdout when omitted"));
}

#[test]
fn rejects_non_numeric_root() -> Result<()> {
    let tmp = TempDir::new()?;
    let bin = tmp.path().join("first.reo");
    let dot = tmp.path().join("first.dot");
    compile_one("F c0", bin.clone())?;
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .current_dir(tmp.path())
        .arg("dot")
        .arg("--root=xyz")
        .arg(bin.as_os_str())
        .arg(dot.as_os_str())
        .assert()
        .failure()
        .code(2);
    Ok(())
}
