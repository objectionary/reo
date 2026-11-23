// SPDX-FileCopyrightText: Copyright (c) 2022-2026 Yegor Bugayenko
// SPDX-License-Identifier: MIT

mod common;

use anyhow::Result;
use reo::Graph;
use sodg::Kind;
use tempfile::TempDir;

#[test]
fn makes_empty_binary() -> Result<()> {
    let tmp = TempDir::new()?;
    let bin = tmp.path().join("e.reo");
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .arg("--verbose")
        .arg("empty")
        .arg(bin.as_os_str())
        .assert()
        .success();
    let g = Graph::load(bin.as_path())?;
    assert_eq!(1, g.len());
    assert_eq!(Kind::Formation, g.kind(0));
    Ok(())
}
