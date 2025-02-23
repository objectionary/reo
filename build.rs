// SPDX-FileCopyrightText: Copyright (c) 2022-2025 Yegor Bugayenko
// SPDX-License-Identifier: MIT

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
