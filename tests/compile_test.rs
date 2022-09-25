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

use anyhow::Result;
use filetime::FileTime;
use predicates::prelude::predicate;
use tempfile::TempDir;

#[test]
fn compiles_everything() -> Result<()> {
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .arg("--verbose")
        .arg("compile")
        .arg("--home=target/eo/gmi/org/eolang/math")
        .arg("target/snippets-math.relf")
        .assert()
        .success();
    Ok(())
}

#[test]
fn skips_compilation_if_file_present() -> Result<()> {
    let tmp = TempDir::new()?;
    let relf = tmp.path().join("foo.relf");
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .arg("compile")
        .arg("--home=target/eo/gmi/org/eolang/io")
        .arg(relf.as_os_str())
        .assert()
        .success();
    let size = std::fs::metadata(&relf)?.len();
    let mtime = FileTime::from_last_modification_time(&std::fs::metadata(&relf)?);
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .arg("compile")
        .arg("--home=target/eo/gmi/org/eolang/io")
        .arg(relf.as_os_str())
        .assert()
        .success();
    assert_eq!(size, std::fs::metadata(&relf)?.len());
    assert_eq!(
        mtime,
        FileTime::from_last_modification_time(&std::fs::metadata(&relf)?)
    );
    Ok(())
}

#[test]
fn fails_when_directory_is_absent() -> Result<()> {
    let path = "/usr/boom";
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .arg("compile")
        .arg(format!("--home={}", path))
        .arg("target/failure.relf")
        .assert()
        .code(1)
        .stderr(predicate::str::contains(format!("Can't access '{}'", path)));
    Ok(())
}
