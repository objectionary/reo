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

use crate::runtime::load_everything;
use anyhow::Result;
use glob::glob;
use reo::Universe;
use std::path::Path;
use sodg::Sodg;
use tempfile::TempDir;

fn all_scripts() -> Result<Vec<String>> {
    let mut scripts = Vec::new();
    for f in glob("sodg-tests/**/*.sodg")? {
        let p = f?;
        scripts.push(p.into_os_string().into_string().unwrap());
    }
    Ok(scripts)
}

#[test]
#[ignore]
fn dataizes_all_sodg_tests() -> Result<()> {
    for path in all_scripts()? {
        let tmp = TempDir::new()?;
        let bin = tmp.path().join("temp.bin");
        assert_cmd::Command::cargo_bin("reo")
            .unwrap()
            .arg("compile")
            .arg(path.clone())
            .arg(bin.as_os_str())
            .assert()
            .success();
        let extra = Sodg::load(bin.as_path())?;
        let mut sodg = load_everything()?;
        sodg.merge(&extra);
        let object = Path::new(&path)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .replace(".sodg", "");
        let mut uni = Universe::from_graph(sodg);
        let ret = uni.dataize(format!("Φ.{}", object).as_str()).unwrap();
        assert!(!ret.is_empty());
    }
    Ok(())
}
