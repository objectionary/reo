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
use reo::universe::Universe;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn load_everything() -> Result<Universe> {
    let relf = Path::new("target/runtime.relf");
    assert_cmd::Command::cargo_bin("reo")?
        .arg("compile")
        .arg("--home=target/eo/gmi")
        .arg(relf.as_os_str())
        .assert()
        .success();
    let uni = Universe::load(relf)?;
    assert!(uni.inconsistencies().is_empty());
    File::create(Path::new("target/runtime-inspect.txt"))?
        .write_all(uni.inspect("Q")?.as_bytes())?;
    Ok(uni)
}
