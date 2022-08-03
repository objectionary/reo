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

use std::path::Path;
use std::{fs, io};

pub struct Xmir {
    xml: String,
}

impl Xmir {
    pub fn from_file(file: &str) -> io::Result<Xmir> {
        return Xmir::from_string(fs::read_to_string(file)?);
    }

    pub fn from_string(xml: String) -> io::Result<Xmir> {
        return Ok(Xmir { xml });
    }

    pub fn to_rust(&self, dir: &Path) -> io::Result<i16> {
        let file = dir.join("test.rs");
        fs::write(file, &self.xml)?;
        return Ok(1);
    }
}

#[cfg(test)]
use tempfile::tempdir;

#[test]
fn translates_fibonacci() {
    let home = tempdir().unwrap();
    let total = Xmir::from_file("target/eo/03-optimize/org/eolang/reo/fibonacci.xmir")
        .unwrap()
        .to_rust(home.path())
        .unwrap();
    assert_eq!(1, total);
}
