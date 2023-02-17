// Copyright (c) 2022-2023 Yegor Bugayenko
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
use log::{debug, info, LevelFilter};
use reo::org::eolang::register;
use reo::Universe;
use simple_logger::SimpleLogger;
use sodg::Script;
use sodg::Sodg;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

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
                    Arg::new("source")
                        .required(true)
                        .value_parser(PathValueParser {})
                        .help("File with .sodg sources to compile")
                        .takes_value(true)
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("target")
                        .required(true)
                        .help("File to save .reo binary")
                        .value_parser(PathValueParser {})
                        .takes_value(true)
                        .action(ArgAction::Set),
                )
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
            let src = subs
                .get_one::<PathBuf>("source")
                .context("Path of .sodg file is required")
                .unwrap();
            debug!("source: {}", src.display());
            if !src.exists() {
                return Err(anyhow!("The file '{}' not found", src.display()));
            }
            let bin = subs
                .get_one::<PathBuf>("target")
                .context("Path of .reo file is required")
                .unwrap();
            debug!("target: {}", bin.display());
            info!(
                "Compiling SODG instructions from '{}' to '{}'",
                src.display(),
                bin.display()
            );
            let mut total = 0;
            let mut g = Sodg::empty();
            let mut s = Script::from_str(fs::read_to_string(src)?.as_str());
            let ints = s
                .deploy_to(&mut g)
                .context(format!("Failed with '{}'", src.display()))?;
            info!("Deployed {ints} instructions from {}", src.display());
            let size = g.save(bin)?;
            info!("The SODG saved to '{}' ({size} bytes)", bin.display());
            total += 1;
            info!("{total} files compiled to {}", bin.display());
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
            info!("Databasing the '{object}' object...");
            let mut uni = Universe::from_graph(g);
            register(&mut uni);
            let ret = uni.dataize(format!("Φ.{}", object).as_str())?.print();
            info!("Datamation result, in {:?} is: {ret}", start.elapsed());
            println!("{ret}");
        }
        _ => unreachable!(),
    }
    Ok(())
}
