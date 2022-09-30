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
mod runtime;

use anyhow::Result;
use glob::glob;
use std::path::Path;
use tempfile::TempDir;
use reo::da;
use reo::universe::Universe;
use crate::runtime::load_everything;

fn all_scripts() -> Result<Vec<String>> {
    let mut scripts = Vec::new();
    for f in glob("gmi-tests/**/*.gmi")? {
        let p = f?;
        scripts.push(p.into_os_string().into_string().unwrap());
    }
    Ok(scripts)
}

#[test]
fn dataizes_all_gmi_tests() -> Result<()> {
    for path in all_scripts()? {
        let tmp = TempDir::new()?;
        let relf = tmp.path().join("temp.relf");
        assert_cmd::Command::cargo_bin("reo")
            .unwrap()
            .arg("compile")
            .arg(format!("--file={}", path))
            .arg(relf.as_os_str())
            .assert()
            .success();
        let extra = Universe::load(relf.as_path())?;
        let mut uni = load_everything()?;
        uni.merge(&extra);
        let object = Path::new(&path).file_name().unwrap().to_str().unwrap().replace(".gmi", "");
        da!(uni, format!("Φ.{}", object));
    }
    Ok(())
}
