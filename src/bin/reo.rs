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
use std::fs::metadata;
use std::path::Path;
use std::time::{Duration, Instant};
use sodg::Script;
use sodg::Sodg;
use timediff::TimeDiff;

pub fn main() -> Result<()> {
    let matches = Command::new("reo")
        .setting(AppSettings::ColorNever)
        .about("SODG-based Virtual Machine for EO Programs")
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
                .about("Compile all instructions into a binary .reo file")
                .arg(
                    Arg::new("eoc")
                        .long("eoc")
                        .required(false)
                        .takes_value(false)
                        .help("Compatibility with eoc command-line toolkit"),
                )
                .arg(
                    Arg::new("binary")
                        .long("binary")
                        .short("b")
                        .required(true)
                        .help("Name of a binary .reo file to create")
                        .takes_value(true)
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("force")
                        .long("force")
                        .short("f")
                        .required(false)
                        .takes_value(false)
                        .help("Compile anyway, even if the binary file is up to date"),
                )
                .arg(
                    Arg::new("file")
                        .required(true)
                        .help("Name of SODG file to use (or directory with files)")
                        .takes_value(true)
                        .action(ArgAction::Set),
                )
        )
        .subcommand(
            Command::new("dataize")
                .setting(AppSettings::ColorNever)
                .about("Dataizes an object")
                .arg(
                    Arg::new("file")
                        .required(true)
                        .help("Name of a binary .reo file to use")
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
    logger.init()?;
    info!(
        "argv: {}",
        std::env::args().collect::<Vec<String>>().join(" ")
    );
    info!("pwd: {}", std::env::current_dir()?.as_path().display());
    let start = Instant::now();
    match matches.subcommand() {
        Some(("compile", subs)) => {
            let bin = Path::new(subs.get_one::<String>("binary")?);
            let mut src;
            if subs.contains_id("eoc") {
                info!("Running in eoc-compatible mode");
                src = Path::new(".eoc/sodg");
            } else {
                src = Path::new(
                    subs.get_one::<String>("file")
                        .context("Path of .sodg file is required (or directory with them)")?
                );
            }
            let mut g = Sodg::empty();
            if metadata(src).unwrap().is_file() {
                if newer(elf, recent) && !subs.contains_id("force") {
                    info!(
                        "The binary file '{}' is up to date ({} bytes), no need to compile (use --force to compile anyway)",
                        bin.display(), fs::metadata(bin)?.len()
                    );
                    return Ok(());
                }
                info!(
                    "Compiling SODG instructions from a single file '{}' into '{}'",
                    src.display(), bin.display()
                );
                let mut s = Script::from_string(fs::read_to_string(src).unwrap());
                let total = s.deploy_to(&mut g)?;
                info!(
                    "Deployed {} SODG instructions in {:?}",
                    total,
                    start.elapsed()
                );
            } else {
                info!("Home requested as '{}'", src.display());
                let full_home =
                    fs::canonicalize(src).context(format!("Can't access '{}'", src.display()))?;
                let cwd = full_home.as_path();
                info!("Home is set to {}", cwd.display());
                if newer_ft(bin, mtime(cwd)?) && !subs.contains_id("force") {
                    info!(
                        "The binary file '{}' ({} bytes) is newer than that directory '{}', no need to compile (use --force to compile anyway)",
                        bin.display(), fs::metadata(bin)?.len(), src.display()
                    );
                    return Ok(());
                }
                info!(
                    "Compiling instructions from the directory '{}' into '{}'",
                    cwd.display(), bin.display()
                );
                g.add(0)?;
                let total = g.setup(cwd)?;
                info!(
                    "Deployed {} SODG instructions in {:?}",
                    total,
                    start.elapsed()
                );
            }
            let size = g.save(bin)?;
            info!(
                "The SODG saved to '{}' ({} bytes)",
                bin.display(),
                size
            );
        }
        Some(("dataize", subs)) => {
            let bin = Path::new(
                subs.value_of("file")
                    .context("Path of .reo file is required")?,
            );
            let object = subs
                .get_one::<String>("object")
                .context("Object name is required")?;
            info!("Deserializing the binary file '{}'", bin.display());
            let mut g = Sodg::load(bin)?;
            info!(
                "Deserialized {} bytes in {:?}",
                fs::metadata(bin)?.len(),
                start.elapsed()
            );
            info!("Dataizing the '{}' object...", object);
            let mut uni = Universe::from_graph(g);
            let ret = uni.dataize(format!("Φ.{}", object).as_str()).as_hex();
            info!("Dataization result, in {:?} is: {}", start.elapsed(), ret);
            println!("{}", ret);
        }
        _ => unreachable!(),
    }
    Ok(())
}
