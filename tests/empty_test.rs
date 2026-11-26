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

#[cfg(target_os = "linux")]
#[test]
fn makes_empty_binary_at_non_utf8_path() -> Result<()> {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    let tmp = TempDir::new()?;
    let bin = tmp.path().join(OsString::from_vec(b"\xffe.reo".to_vec()));
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .arg("empty")
        .arg(bin.as_os_str())
        .assert()
        .success();
    assert_eq!(1, Graph::load(bin.as_path())?.len());
    Ok(())
}
