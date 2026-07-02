// SPDX-FileCopyrightText: Copyright (c) 2022-2026 Yegor Bugayenko
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
        // Generate the EO test fixtures (target/eo/sodg/**) via Maven. We do
        // *not* hard-fail the build when this step doesn't succeed: the pinned
        // EO toolchain pulls objects from objectionary/home that may no longer
        // exist upstream, and Maven/Java may simply be unavailable. Tests that
        // depend on the fixtures detect their absence and skip themselves.
        let generated = Command::new("mvn")
            .arg("--batch-mode")
            .arg("--errors")
            .arg("--debug")
            .arg("--file")
            .arg("test-pom.xml")
            .arg("process-resources")
            .spawn()
            .and_then(|mut child| child.wait())
            .map(|status| status.success())
            .unwrap_or(false);
        if !generated {
            println!(
                "cargo:warning=Maven `process-resources` did not complete, so EO \
                 test fixtures under target/eo/sodg were not (re)generated; tests \
                 that depend on them will be skipped."
            );
        }
        let rt = "target/runtime.eo";
        if Path::new(rt).exists() {
            fs::remove_file(rt).unwrap();
        }
    }
}
