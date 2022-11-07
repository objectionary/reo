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

use anyhow::Context;
use anyhow::Result;
use clap::{crate_version, AppSettings, Arg, ArgAction, Command};
use filetime::FileTime;
use glob::glob;
use log::{info, LevelFilter};
use reo::Universe;
use simple_logger::SimpleLogger;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};
use sodg::Script;
use sodg::Sodg;
use timediff::TimeDiff;

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

pub fn main() -> Result<()> {
    let matches = Command::new("reo")
        .setting(AppSettings::ColorNever)
        .about("SODG to Rust compiler and runner")
        .version(crate_version!())
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .required(false)
                .takes_value(false)
                .help("Print all debug messages"),
        )
        .arg(
            Arg::new("trace")
                .long("trace")
                .required(false)
                .takes_value(false)
                .help("Print all debug AND trace messages (be careful!)"),
        )
        .subcommand_required(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("compile")
                .setting(AppSettings::ColorNever)
                .about("Compile all instructions into a binary .elf file")
                .arg(
                    Arg::new("eoc")
                        .long("eoc")
                        .required(false)
                        .takes_value(false)
                        .help("Compatibility with eoc command-line toolkit"),
                )
                .arg(
                    Arg::new("file")
                        .long("file")
                        .short('f')
                        .required(false)
                        .help("Name of a single .sodg file to work with")
                        .takes_value(true)
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("home")
                        .long("home")
                        .default_value(".")
                        .name("dir")
                        .required(false)
                        .help("Directory with .sodg files")
                        .takes_value(true)
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("elf")
                        .required(true)
                        .help("Name of a binary .elf file to create")
                        .takes_value(true)
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("force")
                        .long("force")
                        .required(false)
                        .takes_value(false)
                        .help("Compile anyway, even if the binary file is up to date"),
                ),
        )
        .subcommand(
            Command::new("dataize")
                .setting(AppSettings::ColorNever)
                .about("Dataizes an object")
                .arg(
                    Arg::new("elf")
                        .long("elf")
                        .required(true)
                        .help("Name of a binary .elf file to use")
                        .takes_value(true)
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("object")
                        .required(true)
                        .help("Fully qualified object name")
                        .takes_value(false)
                        .action(ArgAction::Set),
                )
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("inspect")
                .setting(AppSettings::ColorNever)
                .about("Read a binary universe and print all the details")
                .arg(
                    Arg::new("elf")
                        .required(true)
                        .help("Name of a binary .elf file to use")
                        .takes_value(true)
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("object")
                        .required(true)
                        .help("Fully qualified object name")
                        .takes_value(false)
                        .action(ArgAction::Set),
                )
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("link")
                .setting(AppSettings::ColorNever)
                .about("Take a list of .elf files and join them all into one")
                .arg(
                    Arg::new("elf")
                        .required(true)
                        .help("Name of a binary .elf file to create")
                        .takes_value(true)
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("elfs")
                        .required(true)
                        .multiple(true)
                        .help("Names of a binary .elf files to use as sources")
                        .takes_value(true)
                        .action(ArgAction::Set),
                )
                .arg_required_else_help(true),
        )
        .get_matches();
    let mut logger = SimpleLogger::new().without_timestamps();
    logger = logger.with_level(if matches.contains_id("verbose") {
        LevelFilter::Info
    } else if matches.contains_id("trace") {
        LevelFilter::Trace
    } else {
        LevelFilter::Warn
    });
    logger.init()?;
    info!(
        "argv: {}",
        std::env::args().collect::<Vec<String>>().join(" ")
    );
    info!("pwd: {}", std::env::current_dir()?.as_path().display());
    let start = Instant::now();
    match matches.subcommand() {
        // Some(("compile", subs)) => {
        //     let elf = Path::new(
        //         subs.get_one::<String>("elf")
        //             .context("Path of .elf file is required")?,
        //     );
        //     let mut g = Sodg::empty();
        //     if subs.contains_id("file") {
        //         let recent = Path::new(subs.value_of("file").context("Path of file is required")?);
        //         if newer(elf, recent) && !subs.contains_id("force") {
        //             info!(
        //                 "Relf file '{}' is up to date ({} bytes), no need to compile (use --force to compile anyway)",
        //                 elf.display(), fs::metadata(elf)?.len()
        //             );
        //             return Ok(());
        //         }
        //         info!(
        //             "Deploying SODG instructions from a single file '{}'",
        //             recent.display()
        //         );
        //         let mut s = Script::from_string(fs::read_to_string(recent).unwrap());
        //         let total = s.deploy_to(&mut g)?;
        //         info!(
        //             "Deployed {} SODG instructions in {:?}",
        //             total,
        //             start.elapsed()
        //         );
        //     } else {
        //         let home = if subs.contains_id("eoc") {
        //             info!("Running in eoc-compatible mode");
        //             ".eoc/sodg"
        //         } else {
        //             subs.value_of("dir").unwrap()
        //         };
        //         info!("Home requested as '{}'", home);
        //         let full_home =
        //             fs::canonicalize(home).context(format!("Can't access '{}'", home))?;
        //         let cwd = full_home.as_path();
        //         info!("Home is set to {}", cwd.display());
        //         if newer_ft(elf, mtime(cwd)?) && !subs.contains_id("force") {
        //             info!(
        //                 "Relf file '{}' ({} bytes) is newer than that directory, no need to compile (use --force to compile anyway)",
        //                 elf.display(), fs::metadata(elf)?.len()
        //             );
        //             return Ok(());
        //         }
        //         info!(
        //             "Deploying instructions from a directory '{}'",
        //             cwd.display()
        //         );
        //         g.add(0)?;
        //         let total = g.setup(cwd)?;
        //         info!(
        //             "Deployed {} GMI instructions in {:?}",
        //             total,
        //             start.elapsed()
        //         );
        //     }
        //     let size = g.save(elf)?;
        //     info!(
        //         "The SODG saved to '{}' ({} bytes)",
        //         elf.display(),
        //         size
        //     );
        // }
        // Some(("dataize", subs)) => {
        //     let object = subs
        //         .get_one::<String>("object")
        //         .context("Object name is required")?;
        //     let elf = Path::new(
        //         subs.value_of("elf")
        //             .context("Path of .elf file is required")?,
        //     );
        //     info!("Deserializing a elf file '{}'", elf.display());
        //     let mut g = Sodg::load(elf)?;
        //     info!(
        //         "Deserialized {} bytes in {:?}",
        //         fs::metadata(elf)?.len(),
        //         start.elapsed()
        //     );
        //     info!("Dataizing '{}' object...", object);
        //     let mut uni = Universe::empty();
        //     let ret = uni.dataize(format!("Φ.{}", object).as_str()).as_hex();
        //     info!("Dataization result, in {:?} is: {}", start.elapsed(), ret);
        //     println!("{}", ret);
        // }
        Some(("inspect", subs)) => {
            let elf = Path::new(subs.value_of("elf").unwrap());
            let object = subs
                .get_one::<String>("object")
                .context("Object name is required")?;
            let g = Sodg::load(elf).unwrap();
            info!(
                "Deserialized {} bytes in {:?}",
                fs::metadata(elf).unwrap().len(),
                start.elapsed()
            );
            println!("{}", g.inspect(object.as_str())?);
        }
        // Some(("link", subs)) => {
        //     let target = Path::new(subs.value_of("elf").unwrap());
        //     let mut uni = Universe::load(target).unwrap();
        //     let linked = subs
        //         .values_of("elfs")
        //         .unwrap()
        //         .collect::<Vec<&str>>()
        //         .into_iter()
        //         .map(|f| Sodg::load(Path::new(f)).unwrap())
        //         .inspect(|x| uni.merge(&x))
        //         .count();
        //     let size = g.save(target)?;
        //     info!(
        //         "The SODG made of {} parts saved to '{}' ({} bytes) in {:?}",
        //         linked,
        //         target.display(),
        //         size,
        //         start.elapsed()
        //     );
        // }
        _ => unreachable!(),
    }
    Ok(())
}
