// SPDX-FileCopyrightText: Copyright (c) 2022-2026 Yegor Bugayenko
// SPDX-License-Identifier: MIT

mod common;

use crate::common::compiler::compile_one;
use anyhow::Result;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn inspects_one_binary() -> Result<()> {
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
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .current_dir(tmp.path())
        .arg("inspect")
        .arg("--root=1")
        .arg("--ignore=0")
        .arg("--ignore=42")
        .arg(first.as_os_str())
        .assert()
        .success()
        .stdout(
            predicates::str::contains("Total cells: 2")
                .and(predicates::str::contains("Ignoring: c0, c42"))
                .and(predicates::str::contains("Missed").not()),
        );
    Ok(())
}

#[test]
fn ignores_unknown_ids_when_reporting_missed_vertices() -> Result<()> {
    let tmp = TempDir::new()?;
    let first = tmp.path().join("first.reo");
    compile_one(
        "
        F c0
        F c1
        ",
        first.clone(),
    )?;
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .current_dir(tmp.path())
        .arg("inspect")
        .arg("--root=0")
        .arg("--ignore=99")
        .arg(first.as_os_str())
        .assert()
        .success()
        .stdout(predicates::str::contains("Missed 1: c1"));
    Ok(())
}

#[test]
fn prints_kinds_of_cells() -> Result<()> {
    let tmp = TempDir::new()?;
    let first = tmp.path().join("first.reo");
    compile_one(
        "
        F c0 foo
        D c1 c0 bar
        F c2
        A c3 c1 x c2
        P c0 foo c3
        ",
        first.clone(),
    )?;
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .arg("inspect")
        .arg(first.as_os_str())
        .assert()
        .success()
        .stdout(
            predicates::str::contains("c0 ⟦foo⟧")
                .and(predicates::str::contains(".foo ➞ c3 π(x ↦ …)"))
                .and(predicates::str::contains(".π ➞ c1 π.bar"))
                .and(predicates::str::contains(".π ➞ c0 …")),
        );
    Ok(())
}

#[test]
fn rejects_non_numeric_root() -> Result<()> {
    let tmp = TempDir::new()?;
    let first = tmp.path().join("first.reo");
    compile_one("F c0", first.clone())?;
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .current_dir(tmp.path())
        .arg("inspect")
        .arg("--root=abc")
        .arg(first.as_os_str())
        .assert()
        .failure()
        .code(2);
    Ok(())
}
