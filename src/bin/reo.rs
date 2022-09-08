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
use reo::da;
use reo::gmi::Gmi;
use reo::setup::setup;
use reo::universe::Universe;
use simple_logger::SimpleLogger;
use std::fs;
use std::path::Path;
use std::time::Instant;

fn mtime(dir: &Path) -> Result<FileTime> {
    let mut recent: FileTime = FileTime::from_unix_time(0, 0);
    for f in glob(format!("{}/**/*.gmi", dir.display()).as_str()).unwrap() {
        let mtime = FileTime::from_last_modification_time(&fs::metadata(f.unwrap()).unwrap());
        if mtime > recent {
            recent = mtime;
        }
    }
    Ok(recent)
}

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
                .help("Print all possible debug messages"),
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
                .about("Compile all instructions into a binary .relf file")
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
                        .help("Name of a single .gmi file to work with")
                        .takes_value(true)
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("home")
                        .long("home")
                        .default_value(".")
                        .name("dir")
                        .required(false)
                        .help("Directory with .gmi files")
                        .takes_value(true)
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("relf")
                        .required(true)
                        .help("Name of a binary .relf file to create")
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
                    Arg::new("relf")
                        .long("relf")
                        .required(true)
                        .help("Name of a binary .relf file to use")
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
        .get_matches();
    let mut logger = SimpleLogger::new().without_timestamps();
    logger = logger.with_level(if matches.contains_id("verbose") {
        LevelFilter::Info
    } else if matches.contains_id("trace") {
        LevelFilter::Trace
    } else {
        LevelFilter::Warn
    });
    logger.init().unwrap();
    info!(
        "argv: {}",
        std::env::args().collect::<Vec<String>>().join(" ")
    );
    let start = Instant::now();
    match matches.subcommand() {
        Some(("compile", subs)) => {
            let relf = Path::new(subs.get_one::<String>("relf").unwrap());
            let mut uni = Universe::empty();
            if subs.contains_id("file") {
                let file = Path::new(subs.value_of("file").unwrap());
                let recent = FileTime::from_last_modification_time(&fs::metadata(file).unwrap());
                if relf.exists()
                    && recent < FileTime::from_last_modification_time(&fs::metadata(relf).unwrap())
                    && !subs.contains_id("force")
                {
                    info!(
                        "Relf file '{}' is up to date ({} bytes), no need to compile (use --force to compile anyway)",
                        relf.display(), fs::metadata(relf).unwrap().len()
                    );
                } else {
                    info!(
                        "Deploying instructions from a single file '{}'",
                        file.display()
                    );
                    let total = Gmi::from_file(file).unwrap().deploy_to(&mut uni).unwrap();
                    info!(
                        "Deployed {} GMI instructions in {:?}",
                        total,
                        start.elapsed()
                    );
                }
            } else {
                let home = matches.value_of("dir").unwrap_or_else(|| {
                    if matches.contains_id("eoc") {
                        info!("Running in eoc-compatible mode");
                        ".eoc/gmi"
                    } else {
                        "."
                    }
                });
                info!("Home requested as '{}'", home);
                let full_home = fs::canonicalize(home)
                    .context(format!("Can't access '{}'", home))
                    .unwrap();
                let cwd = full_home.as_path();
                info!("Home is set to {}", cwd.display());
                let recent = mtime(cwd).unwrap();
                if relf.exists()
                    && recent < FileTime::from_last_modification_time(&fs::metadata(relf).unwrap())
                    && !subs.contains_id("force")
                {
                    info!(
                        "Relf file '{}' is up to date ({} bytes), no need to compile (use --force to compile anyway)",
                        relf.display(), fs::metadata(relf).unwrap().len()
                    );
                } else {
                    info!(
                        "Deploying instructions from a directory '{}'",
                        cwd.display()
                    );
                    uni.add(0).unwrap();
                    let total = setup(&mut uni, cwd).unwrap();
                    info!(
                        "Deployed {} GMI instructions in {:?}",
                        total,
                        start.elapsed()
                    );
                }
            }
            let size = uni.save(relf).unwrap();
            info!(
                "The universe saved to '{}' ({} bytes)",
                relf.display(),
                size
            );
        }
        Some(("dataize", subs)) => {
            let object = subs.get_one::<String>("object").unwrap();
            let relf = Path::new(subs.value_of("relf").unwrap());
            info!("Deserializing a relf file '{}'", relf.display());
            let mut uni = Universe::load(relf).unwrap();
            info!(
                "Deserialized {} bytes in {:?}",
                fs::metadata(relf).unwrap().len(),
                start.elapsed()
            );
            info!("Dataizing '{}' object...", object);
            let ret = da!(uni, format!("Φ.{}", object)).as_hex();
            info!("Dataization result, in {:?} is: {}", start.elapsed(), ret);
            println!("{}", ret);
        }
        _ => unreachable!(),
    }
}
