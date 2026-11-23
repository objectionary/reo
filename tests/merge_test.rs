// SPDX-FileCopyrightText: Copyright (c) 2022-2026 Yegor Bugayenko
// SPDX-License-Identifier: MIT

mod common;

use crate::common::compiler::compile_one;
use anyhow::Result;
use reo::{Graph, Universe};
use std::fs;
use tempfile::TempDir;

#[test]
fn merges_two_graphs() -> Result<()> {
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
    let second = tmp.path().join("second.reo");
    compile_one(
        "
        F c0 bar
        F c1
        Δ c1 41-42-43
        P c0 bar c1
        ",
        second.clone(),
    )?;
    let before = fs::metadata(first.clone())?.len();
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .current_dir(tmp.path())
        .arg("merge")
        .arg(first.as_os_str())
        .arg(second.as_os_str())
        .assert()
        .success();
    assert_ne!(before, fs::metadata(first.clone())?.len());
    let mut uni = Universe::from_graph(Graph::load(first.as_path())?)?;
    assert_eq!("привет", uni.dataize("Φ.foo")?.to_utf8()?);
    assert_eq!("ABC", uni.dataize("Φ.bar")?.to_utf8()?);
    Ok(())
}

#[test]
fn merges_phino_output_into_empty_graph() -> Result<()> {
    let tmp = TempDir::new()?;
    let pack = tmp.path().join("pack.reo");
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .arg("empty")
        .arg(pack.as_os_str())
        .assert()
        .success();
    let bin = tmp.path().join("long-phi.reo");
    compile_one(
        fs::read_to_string("quick-tests/07-long-phi.sodg")?.as_str(),
        bin.clone(),
    )?;
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .arg("merge")
        .arg(pack.as_os_str())
        .arg(bin.as_os_str())
        .assert()
        .success();
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .arg("dataize")
        .arg(pack.as_os_str())
        .arg("foo")
        .assert()
        .success()
        .stdout("00-00-00-00-00-00-00-2A\n");
    Ok(())
}
