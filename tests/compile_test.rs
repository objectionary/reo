// SPDX-FileCopyrightText: Copyright (c) 2022-2026 Yegor Bugayenko
// SPDX-License-Identifier: MIT

mod common;

use anyhow::Result;
use glob::glob;
use predicates::prelude::predicate;
use tempfile::TempDir;

#[test]
fn compiles_phino_output_as_is() -> Result<()> {
    let tmp = TempDir::new()?;
    let mut total = 0;
    for f in glob("quick-tests/*.sodg")? {
        let src = f?;
        let bin = tmp
            .path()
            .join(src.with_extension("reo").file_name().unwrap());
        assert_cmd::Command::cargo_bin("reo")
            .unwrap()
            .arg("compile")
            .arg(src.as_os_str())
            .arg(bin.as_os_str())
            .assert()
            .success();
        assert!(bin.exists());
        total += 1;
    }
    assert!(total > 0, "No .sodg files found in quick-tests/");
    Ok(())
}

#[test]
fn dataizes_compiled_phino_output() -> Result<()> {
    let tmp = TempDir::new()?;
    let bin = tmp.path().join("copy-of.reo");
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .arg("compile")
        .arg("quick-tests/06-copy-of.sodg")
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
        .stdout("00-00-00-00-00-00-00-2A\n");
    Ok(())
}

#[test]
fn rejects_positional_attributes() -> Result<()> {
    let tmp = TempDir::new()?;
    let bin = tmp.path().join("positional.reo");
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .arg("compile")
        .arg("quick-errors/05-positional-argument.sodg")
        .arg(bin.as_os_str())
        .assert()
        .code(1)
        .stderr(predicate::str::contains(
            "The positional attribute α0 of c5 is not supported",
        ));
    assert!(!bin.exists());
    Ok(())
}

#[test]
fn compiles_twice_to_the_same_file() -> Result<()> {
    let tmp = TempDir::new()?;
    let bin = tmp.path().join("decorator.reo");
    for _ in 0..2 {
        assert_cmd::Command::cargo_bin("reo")
            .unwrap()
            .arg("--verbose")
            .arg("compile")
            .arg("quick-tests/04-decorator.sodg")
            .arg(bin.as_os_str())
            .assert()
            .success();
    }
    Ok(())
}

#[test]
fn fails_when_file_is_absent() -> Result<()> {
    let path = "/usr/boom-is-absent";
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .arg("compile")
        .arg(path)
        .arg(path)
        .assert()
        .code(1)
        .stderr(predicate::str::contains("not found"));
    Ok(())
}
