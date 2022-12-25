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

use crate::Universe;
use anyhow::{Context, Result};
use glob::glob;
use log::{info, trace};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::BufRead;
use std::path::Path;

/// Compile a file into a
pub fn compile(dir: &Path) -> Result<()> {
}

#[cfg(test)]
use tempfile::TempDir;

#[cfg(test)]
use std::io::Write;
use regex::internal::Input;

#[cfg(test)]
use sodg::Script;

#[test]
fn sets_up_simple_directory() -> Result<()> {
    let tmp = TempDir::new()?;
    File::create(tmp.path().join("foo.sodg"))?.write_all(
        "
        ADD($ν1);
        BIND(ν0, $ν1, foo);
        DATA($ν1, 00-00-00-00-00-00-00-01);
        "
        .as_bytes(),
    )?;
    let mut uni = Universe::empty();
    uni.add();
    uni.setup(tmp.path())?;
    assert_eq!(1, uni.dataize("Φ.foo")?.to_i64()?);
    Ok(())
}

#[test]
fn sets_up_with_subdirectories() -> Result<()> {
    let tmp = TempDir::new()?;
    fs::create_dir(tmp.path().join("abc"))?;
    File::create(tmp.path().join("abc/foo.sodg"))?.write_all(
        "
        ADD($ν1);
        BIND(ν0, $ν1, foo);
        DATA($ν1, 01);
        "
        .as_bytes(),
    )?;
    let mut uni = Universe::empty();
    uni.add();
    uni.setup(tmp.path())?;
    assert_eq!(true, uni.dataize("Φ.abc.foo")?.to_bool()?);
    Ok(())
}
