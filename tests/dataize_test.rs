// SPDX-FileCopyrightText: Copyright (c) 2022-2026 Yegor Bugayenko
// SPDX-License-Identifier: MIT

mod common;

use crate::common::compiler::compile_one;
use anyhow::Result;
use tempfile::TempDir;

#[test]
fn dataizes_simple_program() -> Result<()> {
    let tmp = TempDir::new()?;
    let first = tmp.path().join("first.reo");
    compile_one(
        "
        F c0 foo
        F c1
        Δ c1 d0-bf-d1-80-d0-b8-d0-b2-d0-b5-d1-82
        P c0 foo c1
        ",
        first.clone(),
    )?;
    let dump = tmp.path().join("d.reo");
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .arg("dataize")
        .arg("--dump")
        .arg(dump.as_os_str())
        .arg(first.as_os_str())
        .arg("foo")
        .assert()
        .success()
        .stdout("D0-BF-D1-80-D0-B8-D0-B2-D0-B5-D1-82\n");
    assert!(dump.exists());
    Ok(())
}

#[test]
fn dumps_after_mistake() -> Result<()> {
    let tmp = TempDir::new()?;
    let first = tmp.path().join("first.reo");
    compile_one("F c0", first.clone())?;
    let dump = tmp.path().join("d1.reo");
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .arg("dataize")
        .arg("--dump")
        .arg(dump.as_os_str())
        .arg(first.as_os_str())
        .arg("ν42")
        .assert()
        .failure();
    assert!(dump.exists());
    Ok(())
}
