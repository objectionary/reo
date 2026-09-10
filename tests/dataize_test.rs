// SPDX-FileCopyrightText: Copyright (c) 2022-2026 Yegor Bugayenko
// SPDX-License-Identifier: MIT

mod common;

use crate::common::compiler::compile_one;
use anyhow::Result;
use predicates::prelude::{predicate, PredicateBooleanExt};
use tempfile::TempDir;

#[test]
fn dataizes_simple_program() -> Result<()> {
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
    let dump = tmp.path().join("d.reo");
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .arg("dataize")
        .arg("--dump")
        .arg(dump.as_os_str())
        .arg(first.as_os_str())
        .arg("ν1")
        .assert()
        .success();
    assert!(dump.exists());
    Ok(())
}

#[test]
fn dumps_after_mistake() -> Result<()> {
    let tmp = TempDir::new()?;
    let first = tmp.path().join("first.reo");
    compile_one("ADD(ν0);", first.clone())?;
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

#[test]
fn reports_unregistered_atom_without_panicking() -> Result<()> {
    let tmp = TempDir::new()?;
    let bin = tmp.path().join("missing-atom.reo");
    compile_one(
        "
        ADD(ν0);
        ADD($ν1);
        BIND(ν0, $ν1, foo);
        ADD($ν2);
        BIND($ν1, $ν2, λ);
        PUT($ν2, 6D-69-73-73-69-6E-67);
        ",
        bin.clone(),
    )?;
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .arg("dataize")
        .arg(bin.as_os_str())
        .arg("foo")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Can't find function missing"))
        .stderr(predicate::str::contains("panicked").not());
    Ok(())
}
