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

fn mtime(dir: &Path) -> Result<FileTime> {
    let mut total = 0;
    let mut recent: FileTime = FileTime::from_unix_time(0, 0);
    for f in glob(format!("{}/**/*.sodg", dir.display()).as_str())? {
        let mtime = FileTime::from_last_modification_time(&fs::metadata(f.unwrap()).unwrap());
        if mtime > recent {
            recent = mtime;
        }
        total += 1;
    }
    info!(
        "There are {} .sodg files in {}, the latest modification is {}",
        total,
        dir.display(),
        TimeDiff::to_diff_duration(
            Duration::new(
                (FileTime::now().seconds() - recent.seconds()).try_into().unwrap(),
                0
            )
        ).parse()?
    );
    Ok(recent)
}

/// Returns TRUE if file `f1` is newer than file `f2`.
fn newer(f1: &Path, f2: &Path) -> bool {
    let m2 = if f2.exists() {
        FileTime::from_last_modification_time(&fs::metadata(f2).unwrap())
    } else {
        FileTime::from_unix_time(0, 0)
    };
    newer_ft(f1, m2)
}

/// Returns TRUE if file `f1` is newer than file `f2`.
fn newer_ft(f1: &Path, m2: FileTime) -> bool {
    let m1 = if f1.exists() {
        FileTime::from_last_modification_time(&fs::metadata(f1).unwrap())
    } else {
        FileTime::from_unix_time(0, 0)
    };
    m1 > m2
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
