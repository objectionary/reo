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

use anyhow::Result;
use anyhow::{anyhow, Context};
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
    let mut total = 0;
    let mut recent: FileTime = FileTime::from_unix_time(0, 0);
    for f in glob(format!("{}/**/*.gmi", dir.display()).as_str())? {
        let mtime = FileTime::from_last_modification_time(&fs::metadata(f.unwrap()).unwrap());
        if mtime > recent {
            recent = mtime;
        }
        total += 1;
    }
    info!(
        "There are {} .gmi files in {}, the latest modification is {} minutes ago",
        total,
        dir.display(),
        (FileTime::now().seconds() - recent.seconds()) / 60
    );
    Ok(recent)
}

pub fn main() -> Result<()> {
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
        .subcommand(
            Command::new("inspect")
                .setting(AppSettings::ColorNever)
                .about("Print the universe")
                .arg(
                    Arg::new("relf")
                        .required(true)
                        .help("Name of a binary .relf file to use")
                        .takes_value(true)
                        .action(ArgAction::Set),
                )
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("link")
                .setting(AppSettings::ColorNever)
                .about("Take a list of .relf files and join them all into one")
                .arg(
                    Arg::new("relfs")
                        .required(true)
                        .multiple(true)
                        .help("Name of a binary .relf file to use")
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
        Some(("compile", subs)) => {
            let relf = Path::new(
                subs.get_one::<String>("relf")
                    .context("Path of .relf file is required")?,
            );
            let mut uni = Universe::empty();
            if subs.contains_id("file") {
                let file = Path::new(subs.value_of("file").context("Path of file is required")?);
                let recent = FileTime::from_last_modification_time(&fs::metadata(file)?);
                if relf.exists()
                    && recent < FileTime::from_last_modification_time(&fs::metadata(relf)?)
                    && !subs.contains_id("force")
                {
                    info!(
                        "Relf file '{}' is up to date ({} bytes), no need to compile (use --force to compile anyway)",
                        relf.display(), fs::metadata(relf)?.len()
                    );
                } else {
                    info!(
                        "Deploying instructions from a single file '{}'",
                        file.display()
                    );
                    let total = Gmi::from_file(file)?.deploy_to(&mut uni)?;
                    info!(
                        "Deployed {} GMI instructions in {:?}",
                        total,
                        start.elapsed()
                    );
                }
            } else {
                let home = subs.value_of("dir").unwrap_or_else(|| {
                    if subs.contains_id("eoc") {
                        info!("Running in eoc-compatible mode");
                        ".eoc/gmi"
                    } else {
                        "."
                    }
                });
                info!("Home requested as '{}'", home);
                if !Path::new(home).exists() {
                    return Err(anyhow!("Directory '{}' doesn't exist", home));
                }
                let full_home =
                    fs::canonicalize(home).context(format!("Can't access '{}'", home))?;
                let cwd = full_home.as_path();
                info!("Home is set to {}", cwd.display());
                let recent = mtime(cwd)?;
                if relf.exists()
                    && recent < FileTime::from_last_modification_time(&fs::metadata(relf)?)
                    && !subs.contains_id("force")
                {
                    info!(
                        "Relf file '{}' is up to date ({} bytes), no need to compile (use --force to compile anyway)",
                        relf.display(), fs::metadata(relf)?.len()
                    );
                } else {
                    info!(
                        "Deploying instructions from a directory '{}'",
                        cwd.display()
                    );
                    uni.add(0)?;
                    let total = setup(&mut uni, cwd)?;
                    info!(
                        "Deployed {} GMI instructions in {:?}",
                        total,
                        start.elapsed()
                    );
                }
            }
            let size = uni.save(relf)?;
            info!(
                "The universe saved to '{}' ({} bytes)",
                relf.display(),
                size
            );
        }
        Some(("dataize", subs)) => {
            let object = subs
                .get_one::<String>("object")
                .context("Object name is required")?;
            let relf = Path::new(
                subs.value_of("relf")
                    .context("Path of .relf file is required")?,
            );
            info!("Deserializing a relf file '{}'", relf.display());
            let mut uni = Universe::load(relf)?;
            info!(
                "Deserialized {} bytes in {:?}",
                fs::metadata(relf)?.len(),
                start.elapsed()
            );
            info!("Dataizing '{}' object...", object);
            let ret = da!(uni, format!("Φ.{}", object)).as_hex();
            info!("Dataization result, in {:?} is: {}", start.elapsed(), ret);
            println!("{}", ret);
        }
        Some(("inspect", subs)) => {
            let relf = Path::new(subs.value_of("relf").unwrap());
            let uni = Universe::load(relf).unwrap();
            info!(
                "Deserialized {} bytes in {:?}",
                fs::metadata(relf).unwrap().len(),
                start.elapsed()
            );
            let json = serde_json::to_string_pretty(&uni).unwrap();
            println!("Universe is: {}", json);
        }
        Some(("link", subs)) => {
            let args: Vec<&str> = subs.values_of("relfs").unwrap().collect();
            let target = Path::new(args[0]);
            let mut universe = Universe::load(target).unwrap();
            let mut relfs = args.clone();
            relfs.retain(|&x| x != args[0]);
            let universes: Vec<Universe> = relfs
                .into_iter()
                .map(|x| Universe::load(Path::new(x)).unwrap())
                .collect();
            for uni in universes {
                universe.merge(&uni);
            }
            let size = universe.save(target)?;
            info!(
                "The universe saved to '{}' ({} bytes)",
                target.display(),
                size
            );
        }
        _ => unreachable!(),
    }
    Ok(())
}
