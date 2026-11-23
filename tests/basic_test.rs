// SPDX-FileCopyrightText: Copyright (c) 2022-2026 Yegor Bugayenko
// SPDX-License-Identifier: MIT

mod common;

use anyhow::Result;
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

#[test]
fn dataizes_simple_sodg() -> Result<()> {
    let tmp = TempDir::new()?;
    let src = tmp.path().join("foo.sodg");
    let bin = tmp.path().join("foo.reo");
    File::create(src.clone())?.write_all(
        "
        F c0 foo
        F c1
        Δ c1 ff-ff
        P c0 foo c1
        "
        .as_bytes(),
    )?;
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .current_dir(tmp.path())
        .arg("--trace")
        .arg("compile")
        .arg(src.as_os_str())
        .arg(bin.as_os_str())
        .assert()
        .success();
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .arg("dataize")
        .arg(bin.as_os_str())
        .arg("foo")
        .assert()
        .success()
        .stdout("FF-FF\n");
    Ok(())
}
