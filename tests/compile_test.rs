// Copyright (c) 2022 Yegor Bugayenko
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included
// in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NON-INFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

mod common;

use anyhow::Result;
use filetime::FileTime;
use predicates::prelude::predicate;
use tempfile::TempDir;

#[test]
fn compiles_everything() -> Result<()> {
    let tmp = TempDir::new()?;
    let target = tmp.path().join("target");
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .arg("--verbose")
        .arg("compile")
        .arg("target/eo/sodg")
        .arg(target.as_os_str())
        .assert()
        .success();
    assert!(target.join("org/eolang/int.reo").exists());
    Ok(())
}

#[test]
fn skips_compilation_if_file_present() -> Result<()> {
    let tmp = TempDir::new()?;
    let target = tmp.path().join("target");
    let bin = target.join("org/eolang/int.reo");
    let mut first = None;
    for _ in 0..2 {
        assert_cmd::Command::cargo_bin("reo")
            .unwrap()
            .arg("--verbose")
            .arg("compile")
            .arg("target/eo/sodg")
            .arg(target.as_os_str())
            .assert()
            .success();
        let now = FileTime::from_last_modification_time(&std::fs::metadata(&bin)?);
        if let Some(before) = first {
            assert_eq!(before, now);
        } else {
            first = Some(now)
        }
    }
    Ok(())
}

#[test]
fn fails_when_directory_is_absent() -> Result<()> {
    let path = "/usr/boom";
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
