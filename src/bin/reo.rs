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

extern crate reo;

use std::fs;
use std::fs::File;
use std::path::Path;
use clap::{AppSettings, Arg, ArgAction, Command, crate_version};
use log::{debug, LevelFilter};
use predicates::prelude::predicate;
use simple_logger::SimpleLogger;

pub fn main() {
    let matches = Command::new("reo")
        .setting(AppSettings::ColorNever)
        .about("GMI to Rust compiler and runner")
        .version(crate_version!())
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .required(false)
                .takes_value(false)
                .help("Print all possible debug messages")
        )
        .arg(
            Arg::new("file")
                .long("file")
                .short('f')
                .required(false)
                .help("Name of a single .gmi file to work with")
                .takes_value(true)
                .action(ArgAction::Set)
        )
        .arg(
            Arg::new("home")
                .long("home")
                .required(false)
                .help("Directory with .gmi files")
                .takes_value(true)
                .action(ArgAction::Set)
        )
        .subcommand_required(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("dataize")
                .setting(AppSettings::ColorNever)
                .about("Dataizes an object")
                .arg(
                    Arg::new("object")
                        .required(true)
                        .help("Fully qualified object name")
                        .takes_value(false)
                        .action(ArgAction::Set)
                )
                .arg_required_else_help(true)
        )
        .get_matches();
    let mut logger = SimpleLogger::new().without_timestamps();
    if matches.contains_id("verbose") {
        logger = logger.with_level(LevelFilter::Trace);
    }
    logger.init().unwrap();
    match matches.subcommand() {
        Some(("dataize", subs)) => {
            let object = subs.get_one::<String>("object").unwrap();
            let home = matches.value_of("home").unwrap_or(".");
            let full_home = fs::canonicalize(home).unwrap();
            let cwd = full_home.as_path();
            let mut uni = Universe::empty();
            debug!("Home is set to {}", cwd.display());
            let mut total = 0;
            if matches.contains_id("file") {
                let file = Path::new(matches.value_of("file").unwrap());
                debug!("Deploying instructions from a single file {}", file.display());
                total += Gmi::from_file(file).unwrap().deploy_to(&mut uni).unwrap();
            } else {
                debug!("Deploying instructions from a directory {}", cwd.display());
                uni.add(0).unwrap();
                total += setup(&mut uni, cwd).unwrap();
            }
            debug!("Deployed {} GMI instructions", total);
            debug!("Dataizing '{}' object...", object);
            let ret = da!(uni, format!("Φ.{}", object)).as_hex();
            debug!("Dataization result is: {}", ret);
            println!("{}", ret);
        }
        _ => unreachable!(),
    }
}

#[test]
fn prints_help() {
    assert_cmd::Command::cargo_bin("reo").unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("GMI to Rust"))
        .stdout(predicate::str::contains("--home"));
}

#[cfg(test)]
use tempfile::TempDir;
use reo::da;
use reo::setup::setup;
use reo::universe::Universe;
use anyhow::Result;
use std::io::Write;
use reo::gmi::Gmi;
use glob::glob;

#[test]
fn dataizes_simple_gmi() -> Result<()> {
    let tmp = TempDir::new()?;
    File::create(tmp.path().join("foo.gmi"))?.write_all(
        "
        ADD('$ν1');
        BIND('$ε2', 'ν0', '$ν1', 'foo');
        DATA('$ν1', 'ff ff');
        ".as_bytes()
    )?;
    assert_cmd::Command::cargo_bin("reo").unwrap()
        .arg(format!("--home={}", tmp.path().display()))
        .arg("dataize")
        .arg("foo")
        .assert()
        .success()
        .stdout("ff-ff\n");
    Ok(())
}

#[test]
fn dataizes_all_gmi_tests() -> Result<()> {
    for f in glob("gmi-tests/*.gmi")? {
        let p = f?;
        let path = p.as_path();
        assert_cmd::Command::cargo_bin("reo").unwrap()
            .arg(format!("--file={}", path.display()))
            .arg("--verbose")
            .arg("dataize")
            .arg("foo")
            .assert()
            .success()
            .stdout(predicate::str::contains("Dataization result is: "));
    }
    Ok(())
}
