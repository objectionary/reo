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

use anyhow::{Context, Result};
use glob::glob;
use predicates::prelude::predicate;
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

#[test]
fn dataizes_simple_gmi() -> Result<()> {
    let tmp = TempDir::new()?;
    File::create(tmp.path().join("foo.gmi"))?.write_all(
        "
        ADD('$ν1');
        BIND('$ε2', 'ν0', '$ν1', 'foo');
        DATA('$ν1', 'ff ff');
        "
        .as_bytes(),
    )?;
    let relf = tmp.path().join("temp.relf");
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .current_dir(tmp.path())
        .arg("--verbose")
        .arg("compile")
        .arg(format!("--home={}", tmp.path().display()))
        .arg(relf.as_os_str())
        .assert()
        .success();
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .arg("dataize")
        .arg(format!("--relf={}", relf.display()))
        .arg("foo")
        .assert()
        .success()
        .stdout("FF-FF\n");
    Ok(())
}

#[test]
fn dataizes_in_eoc_mode() -> Result<()> {
    let tmp = TempDir::new()?;
    let dir = tmp.path().join(".eoc").join("gmi");
    fsutils::mkdir(
        dir.to_str()
            .context(format!("Broken path {}", dir.display()))?,
    );
    File::create(dir.join("foo.gmi"))?.write_all(
        "
        ADD('$ν1');
        BIND('$ε1', 'ν0', '$ν1', 'foo');
        DATA('$ν1', 'ca fe');
        "
        .as_bytes(),
    )?;
    let relf = tmp.path().join("temp.relf");
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .current_dir(tmp.path())
        .arg("compile")
        .arg("--eoc")
        .arg(relf.as_os_str())
        .assert()
        .success();
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .current_dir(tmp.path())
        .arg("dataize")
        .arg(format!("--relf={}", relf.display()))
        .arg("foo")
        .assert()
        .success()
        .stdout("CA-FE\n");
    Ok(())
}

#[test]
fn dataizes_all_gmi_tests() -> Result<()> {
    let tmp = TempDir::new()?;
    let relf = tmp.path().join("temp.relf");
    for f in glob("gmi-tests/*.gmi")? {
        let p = f?;
        let path = p.as_path();
        assert_cmd::Command::cargo_bin("reo")
            .unwrap()
            .arg("compile")
            .arg(format!("--file={}", path.display()))
            .arg(relf.as_os_str())
            .assert()
            .success();
        assert_cmd::Command::cargo_bin("reo")
            .unwrap()
            .arg("--verbose")
            .arg("dataize")
            .arg(format!("--relf={}", relf.display()))
            .arg("foo")
            .assert()
            .success()
            .stdout(predicate::str::contains("Dataization result"));
    }
    Ok(())
}
