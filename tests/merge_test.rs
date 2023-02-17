// Copyright (c) 2022-2023 Yegor Bugayenko
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
use reo::Universe;
use sodg::Sodg;
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

#[test]
fn merges_two_graphs() -> Result<()> {
    let tmp = TempDir::new()?;
    fsutils::mkdir(tmp.path().join("src").into_os_string().to_str().unwrap());
    fsutils::mkdir(tmp.path().join("reo").into_os_string().to_str().unwrap());
    File::create(tmp.path().join("src/foo.sodg"))?.write_all(
        "
        ADD(ν0);
        ADD($ν1);
        BIND(ν0, $ν1, foo);
        PUT($ν1, d0-bf-d1-80-d0-b8-d0-b2-d0-b5-d1-82);
        "
        .as_bytes(),
    )?;
    File::create(tmp.path().join("src/bar.sodg"))?.write_all(
        "
        ADD(ν0);
        ADD($ν1);
        BIND(ν0, $ν1, bar);
        PUT($ν1, 40-41-42);
        "
        .as_bytes(),
    )?;
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .current_dir(tmp.path())
        .arg("compile")
        .arg(tmp.path().join("src").as_os_str())
        .arg(tmp.path().join("reo").as_os_str())
        .assert()
        .success();
    let app = tmp.path().join("app.reo");
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .current_dir(tmp.path())
        .arg("merge")
        .arg(app.as_os_str())
        .arg(tmp.path().join("reo").as_os_str())
        .assert()
        .success();
    let mut uni = Universe::from_graph(Sodg::load(app.as_path())?);
    assert_eq!("ff", uni.dataize("Φ.foo")?.to_utf8()?);
    assert_eq!("ff", uni.dataize("Φ.bar")?.to_utf8()?);
    Ok(())
}
