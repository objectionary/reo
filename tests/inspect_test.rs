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
use predicates::prelude::predicate;
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

#[test]
fn inspect_existing() -> Result<()> {
    let tmp = TempDir::new()?;
    File::create(tmp.path().join("foo.g"))?.write_all(
        "
        ADD($ν1);
        BIND(ν0, $ν1, foo);
        DATA($ν1, ff-ff);
        "
        .as_bytes(),
    )?;
    let elf = tmp.path().join("temp.elf");
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .current_dir(tmp.path())
        .arg("compile")
        .arg(format!("--home={}", tmp.path().display()))
        .arg(elf.as_os_str())
        .assert()
        .success();
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .arg("inspect")
        .arg(elf.as_os_str())
        .arg("Q")
        .assert()
        .success()
        .stdout(predicate::str::contains(".foo ➞ ν"));
    Ok(())
}

#[test]
fn inspect_nonexisting() -> Result<()> {
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .arg("inspect")
        .arg("broken-file-name.elf")
        .arg("foo")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Can't read from "));
    Ok(())
}
