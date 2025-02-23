// SPDX-FileCopyrightText: Copyright (c) 2022-2025 Yegor Bugayenko
// SPDX-License-Identifier: MIT

mod common;

use crate::common::compiler::compile_one;
use anyhow::Result;
use tempfile::TempDir;

#[test]
fn prints_dot() -> Result<()> {
    let tmp = TempDir::new()?;
    let bin = tmp.path().join("first.reo");
    let dot = tmp.path().join("first.dot");
    compile_one(
        "
        ADD(ν0);
        ADD($ν1);
        BIND(ν0, $ν1, foo);
        ADD($ν2);
        BIND($ν1, $ν2, Δ);
        PUT($ν2, d0-bf-d1-80-d0-b8-d0-b2-d0-b5-d1-82);
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
    assert!(dot.exists());
    Ok(())
}
