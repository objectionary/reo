// SPDX-FileCopyrightText: Copyright (c) 2022-2025 Yegor Bugayenko
// SPDX-License-Identifier: MIT

mod common;

use crate::common::compiler::compile_one;
use anyhow::Result;
use tempfile::TempDir;

#[test]
fn inspects_one_binary() -> Result<()> {
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
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .current_dir(tmp.path())
        .arg("inspect")
        .arg("--root=2")
        .arg("--ignore=1")
        .arg("--ignore=42")
        .arg(first.as_os_str())
        .assert()
        .success();
    Ok(())
}
