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
use clap::builder::TypedValueParser;
use clap::ErrorKind::EmptyValue;
use clap::{crate_version, AppSettings, Arg, ArgAction, Command};
use filetime::FileTime;
use glob::glob;
use log::{debug, info, LevelFilter};
use reo::org::eolang::register;
use reo::Universe;
use simple_logger::SimpleLogger;
use sodg::Script;
use sodg::Sodg;
use std::collections::HashMap;
use std::fs;
use std::fs::metadata;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Returns TRUE if file `f1` is newer than file `f2`.
fn newer(f1: &Path, f2: &Path) -> bool {
    let m2 = if f2.exists() {
        FileTime::from_last_modification_time(&metadata(f2).unwrap())
    } else {
        FileTime::from_unix_time(0, 0)
    };
    newer_ft(f1, m2)
}

/// Returns TRUE if file `f1` is newer than file `f2`.
fn newer_ft(f1: &Path, m2: FileTime) -> bool {
    let m1 = if f1.exists() {
        FileTime::from_last_modification_time(&metadata(f1).unwrap())
    } else {
        FileTime::from_unix_time(0, 0)
    };
    m1 > m2
}

#[derive(Copy, Clone, Debug)]
struct PathValueParser {}

impl TypedValueParser for PathValueParser {
    type Value = PathBuf;
    fn parse_ref(
        &self,
        _cmd: &Command,
        _arg: Option<&Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<PathBuf, clap::Error> {
        if value.is_empty() {
            return Err(clap::Error::raw(EmptyValue, "Can't be empty"));
        }
        let path = Path::new(value.to_str().unwrap());
        let abs = if path.exists() {
            fs::canonicalize(path).unwrap()
        } else {
            path.to_path_buf()
        };
        Ok(abs)
    }
}

pub fn main() -> Result<()> {
    let matches = Command::new("reo")
        .setting(AppSettings::ColorNever)
        .about("SODG-based Virtual Machine for EO Programs")
        .version(crate_version!())
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .required(false)
                .help("Print all debug messages")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("trace")
                .long("trace")
                .required(false)
                .help("Print all debug AND trace messages (be careful!)")
                .action(ArgAction::Set),
        )
        .subcommand_required(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("compile")
                .setting(AppSettings::ColorNever)
                .about("Compile .sodg files into binary .reo files")
                .arg(
                    Arg::new("sources")
                        .required(true)
                        .value_parser(PathValueParser {})
                        .help("Directory with .sodg files to compile")
                        .takes_value(true)
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("target")
                        .required(true)
                        .help("Directory with .reo binary files to create")
                        .value_parser(PathValueParser {})
                        .takes_value(true)
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("force")
                        .long("force")
                        .short('f')
                        .required(false)
                        .takes_value(false)
                        .help("Compile anyway, even if a binary file is up to date"),
                ),
        )
        .subcommand(
            Command::new("merge")
                .setting(AppSettings::ColorNever)
                .about("Merge binary .reo files into a single .reo file")
                .arg(
                    Arg::new("file")
                        .required(true)
                        .value_parser(PathValueParser {})
                        .help("Name of a binary .reo file to create")
                        .takes_value(true)
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("target")
                        .required(true)
                        .value_parser(PathValueParser {})
                        .help("Directory with .reo binary files")
                        .takes_value(true)
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("force")
                        .long("force")
                        .short('f')
                        .required(false)
                        .takes_value(false)
                        .help("Merge anyway, even if a binary file is up to date"),
                ),
        )
        .subcommand(
            Command::new("dataize")
                .setting(AppSettings::ColorNever)
                .about("Dataize an object in .reo file")
                .arg(
                    Arg::new("file")
                        .required(true)
                        .value_parser(PathValueParser {})
                        .help("Name of a binary .reo file to use")
                        .takes_value(true)
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("object")
                        .required(true)
                        .help("Fully qualified object name")
                        .action(ArgAction::Set),
                )
                .arg_required_else_help(true),
        )
        .get_matches();
    let mut logger = SimpleLogger::new().without_timestamps();
    logger = logger.with_level(if matches.get_flag("verbose") {
        LevelFilter::Info
    } else if matches.contains_id("trace") {
        LevelFilter::Trace
    } else {
        LevelFilter::Warn
    });
    logger.init()?;
    debug!(
        "argv: {}",
        std::env::args().collect::<Vec<String>>().join(" ")
    );
    debug!("pwd: {}", std::env::current_dir()?.as_path().display());
    let start = Instant::now();
    match matches.subcommand() {
        Some(("compile", subs)) => {
            let sources = subs
                .get_one::<PathBuf>("sources")
                .context("Path of directory with .sodg files is required")
                .unwrap();
            debug!("sources: {}", sources.display());
            if !sources.exists() {
                return Err(anyhow!("The directory '{}' not found", sources.display()));
            }
            let target = subs
                .get_one::<PathBuf>("target")
                .context("Path of directory with .reo files is required")
                .unwrap();
            debug!("target: {}", target.display());
            let mut job = HashMap::new();
            if sources.is_dir() {
                debug!("the sources is a directory: {}", sources.display());
                if !target.exists()
                    && fsutils::mkdir(target.clone().into_os_string().to_str().unwrap())
                {
                    info!("Directory created: '{}'", target.display());
                }
                for f in glob(format!("{}/**/*.sodg", sources.display()).as_str())? {
                    let src = f?;
                    if src.is_dir() {
                        continue;
                    }
                    let rel = src
                        .as_path()
                        .strip_prefix(sources.as_path())?
                        .with_extension("reo");
                    let bin = target.join(rel);
                    job.insert(src, bin);
                }
            } else {
                debug!("the sources is a single file: {}", sources.display());
                job.insert((*sources).clone(), (*target).clone());
            }
            let mut total = 0;
            for (src, bin) in &job {
                let parent = bin
                    .parent()
                    .context(format!("Can't get parent of {}", bin.display()))?;
                if fsutils::mkdir(parent.to_str().unwrap()) {
                    info!("Directory created: '{}'", parent.display());
                }
                debug!("bin: {}", bin.display());
                if newer(bin, src) && !subs.contains_id("force") {
                    info!(
                        "The binary file '{}' is up to date ({} bytes), no need to compile the source file '{}' (use --force to compile anyway)",
                        bin.display(), fs::metadata(bin)?.len(), src.display()
                    );
                    continue;
                }
                let mut g = Sodg::empty();
                info!(
                    "Compiling SODG instructions from '{}' to '{}'",
                    src.display(),
                    bin.display()
                );
                let mut s = Script::from_str(fs::read_to_string(src)?.as_str());
                let ints = s
                    .deploy_to(&mut g)
                    .context(format!("Failed with '{}'", src.display()))?;
                info!("Deployed {ints} instructions from {}", src.display());
                let size = g.save(bin)?;
                info!("The SODG saved to '{}' ({size} bytes)", bin.display());
                total += 1;
            }
            info!("{total} files compiled to {}", target.display());
        }
        Some(("dataize", subs)) => {
            let bin = subs
                .get_one::<PathBuf>("file")
                .context("Path of .reo file is required")
                .unwrap();
            debug!("bin: {}", bin.display());
            if !bin.exists() {
                return Err(anyhow!("The file '{}' doesn't exist", bin.display()));
            }
            let object = subs
                .get_one::<String>("object")
                .context("Object name is required")?;
            debug!("object: {}", object);
            info!("Deserializing the binary file '{}'", bin.display());
            let g = Sodg::load(bin.as_path())?;
            info!(
                "Deserialized {} bytes in {:?}",
                fs::metadata(bin)?.len(),
                start.elapsed()
            );
            info!("Dataizing the '{object}' object...");
            let mut uni = Universe::from_graph(g);
            register(&mut uni);
            let ret = uni.dataize(format!("Φ.{}", object).as_str())?.print();
            info!("Dataization result, in {:?} is: {ret}", start.elapsed());
            println!("{ret}");
        }
        _ => unreachable!(),
    }
    Ok(())
}
