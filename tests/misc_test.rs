// SPDX-FileCopyrightText: Copyright (c) 2022-2025 Yegor Bugayenko
// SPDX-License-Identifier: MIT

mod common;

use predicates::prelude::predicate;
use predicates::prelude::*;

#[test]
fn prints_help() {
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Virtual Machine").and(predicate::str::contains("--help")),
        );
}

#[test]
fn prints_version() {
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .arg("--version")
        .assert()
        .success();
}
