// Copyright (c) 2022-2025 Yegor Bugayenko
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
use predicates::prelude::predicate;
use tempfile::TempDir;

#[test]
fn compiles_int() -> Result<()> {
    let tmp = TempDir::new()?;
    let bin = tmp.path().join("int.reo");
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .arg("--verbose")
        .arg("compile")
        .arg("target/eo/sodg/org/eolang/int.sodg")
        .arg(bin.as_os_str())
        .assert()
        .success();
    assert!(bin.exists());
    Ok(())
}

#[test]
fn skips_compilation_if_file_present() -> Result<()> {
    let tmp = TempDir::new()?;
    let bin = tmp.path().join("float.reo");
    for _ in 0..2 {
        assert_cmd::Command::cargo_bin("reo")
            .unwrap()
            .arg("--verbose")
            .arg("compile")
            .arg("target/eo/sodg/org/eolang/tuple.sodg")
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
