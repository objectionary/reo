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

use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    if std::env::var("PROFILE").unwrap() == "debug" {
        println!("cargo:rerun-if-changed=eo-tests");
        println!("cargo:rerun-if-changed=build.rs");
        println!("cargo:rerun-if-changed=test-pom.xml");
        println!("cargo:rerun-if-changed=target/eo/sodg");
        assert!(Command::new("mvn")
            .arg("--batch-mode")
            .arg("--errors")
            .arg("--debug")
            .arg("--file")
            .arg("test-pom.xml")
            .arg("process-resources")
            .spawn()
            .unwrap()
            .wait()
            .unwrap()
            .success());
        let rt = "target/runtime.eo";
        if Path::new(rt).exists() {
            fs::remove_file(rt).unwrap();
        }
    }
}
