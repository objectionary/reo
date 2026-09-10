// SPDX-FileCopyrightText: Copyright (c) 2022-2026 Yegor Bugayenko
// SPDX-License-Identifier: MIT

mod common;

use anyhow::Result;
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
    assert!(bin.exists());
    Ok(())
}

#[cfg(unix)]
#[test]
fn accepts_non_utf_output_path() -> Result<()> {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let tmp = TempDir::new()?;
    let bin = tmp.path().join(OsString::from_vec(vec![
        b'o', b'u', b't', b'-', 0xff, b'.', b'r', b'e', b'o',
    ]));
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .arg("empty")
        .arg(bin.as_os_str())
        .assert()
        .success();
    assert!(bin.exists());
    Ok(())
}
