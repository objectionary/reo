// SPDX-FileCopyrightText: Copyright (c) 2022-2025 Yegor Bugayenko
// SPDX-License-Identifier: MIT

mod common;

use crate::common::compiler::compile_one;
use anyhow::Result;
use log::debug;
use reo::Universe;
use sodg::Sodg;
use std::fs;
use tempfile::TempDir;

#[test]
fn merges_two_graphs() -> Result<()> {
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
    let second = tmp.path().join("second.reo");
    compile_one(
        "
        ADD(ν0);
        ADD($ν1);
        BIND(ν0, $ν1, bar);
        ADD($ν2);
        BIND($ν1, $ν2, Δ);
        PUT($ν2, 41-42-43);
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
    let g = Sodg::load(first.as_path())?;
    debug!("{g:?}");
    let mut uni = Universe::from_graph(g);
    assert_eq!("привет", uni.dataize("Φ.foo")?.to_utf8()?);
    assert_eq!("ABC", uni.dataize("Φ.bar")?.to_utf8()?);
    Ok(())
}
