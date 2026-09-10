// SPDX-FileCopyrightText: Copyright (c) 2022-2026 Yegor Bugayenko
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

#[cfg(unix)]
#[test]
fn supports_non_utf_snapshot_path() -> anyhow::Result<()> {
    use reo::Universe;
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    use tempfile::TempDir;

    let tmp = TempDir::new()?;
    let snapshots = tmp
        .path()
        .join(OsString::from_vec(vec![b's', b'n', b'a', b'p', b'-', 0xff]));
    let mut universe = Universe::empty();
    universe.add();
    let mut universe = universe.with_snapshots(&snapshots);
    assert!(universe.dataize("Φ.missing").is_err());
    assert!(snapshots.exists());
    Ok(())
}
