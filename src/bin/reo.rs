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
use itertools::Itertools;
use log::{debug, info, LevelFilter};
use reo::org::eolang::register;
use reo::Universe;
use simple_logger::SimpleLogger;
use sodg::Script;
use sodg::Sodg;
use std::collections::HashSet;
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
                .action(ArgAction::SetTrue),
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
                ),
        )
        .subcommand(
            Command::new("empty")
                .setting(AppSettings::ColorNever)
                .about("Make an empty .reo file")
                .arg(
                    Arg::new("target")
                        .required(true)
                        .help("File to save .reo binary")
                        .value_parser(PathValueParser {})
                        .takes_value(true)
                        .action(ArgAction::Set),
                ),
        )
        .subcommand(
            Command::new("merge")
                .setting(AppSettings::ColorNever)
                .about("Merge .reo file into an existing .reo file")
                .arg(
                    Arg::new("target")
                        .required(true)
                        .value_parser(PathValueParser {})
                        .help("Path of .reo file to merge into")
                        .takes_value(true)
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("source")
                        .required(true)
                        .value_parser(PathValueParser {})
                        .help("Path of .reo file being merged")
                        .takes_value(true)
                        .action(ArgAction::Set),
                ),
        )
        .subcommand(
            Command::new("inspect")
                .setting(AppSettings::ColorNever)
                .about("Print all visible information from a binary .reo file")
                .arg(
                    Arg::new("bin")
                        .required(true)
                        .value_parser(PathValueParser {})
                        .help("Path of .reo file to inspect")
                        .takes_value(true)
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("root")
                        .long("root")
                        .required(false)
                        .default_value("0")
                        .help("The ID of the root vertex")
                        .action(ArgAction::Set),
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
        .subcommand(
            Command::new("dot")
                .setting(AppSettings::ColorNever)
                .about("Turn binary .reo file to .dot file")
                .arg(
                    Arg::new("bin")
                        .required(true)
                        .value_parser(PathValueParser {})
                        .help("Name of a binary .reo file to use")
                        .takes_value(true)
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("dot")
                        .required(true)
                        .value_parser(PathValueParser {})
                        .help("Name of a .dot file to create")
                        .takes_value(true)
                        .action(ArgAction::Set),
                )
                .arg_required_else_help(true),
        )
        .get_matches();
    let mut logger = SimpleLogger::new().without_timestamps();
    logger = logger.with_level(if matches.get_flag("verbose") {
        LevelFilter::Info
    } else if matches.get_flag("trace") {
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
            let mut g = Sodg::empty();
            let mut s = Script::from_str(fs::read_to_string(src)?.as_str());
            let ints = s
                .deploy_to(&mut g)
                .context(format!("Failed with '{}'", src.display()))?;
            info!("Deployed {ints} instructions from {}", src.display());
            let size = g.save(bin)?;
            info!("The SODG saved to '{}' ({size} bytes)", bin.display());
        }
        Some(("empty", subs)) => {
            let bin = subs
                .get_one::<PathBuf>("target")
                .context("Path of .reo file is required")
                .unwrap();
            debug!("target: {}", bin.display());
            let mut g = Sodg::empty();
            let size = g.save(bin)?;
            info!("The SODG saved to '{}' ({size} bytes)", bin.display());
        }
        Some(("merge", subs)) => {
            let target = subs
                .get_one::<PathBuf>("target")
                .context("Path of target .reo file is required")
                .unwrap();
            debug!("target: {}", target.display());
            if !target.exists() {
                return Err(anyhow!("The file '{}' not found", target.display()));
            }
            let source = subs
                .get_one::<PathBuf>("source")
                .context("Path of source .reo file is required")
                .unwrap();
            debug!("source: {}", source.display());
            if !source.exists() {
                return Err(anyhow!("The file '{}' not found", source.display()));
            }
            info!("Merging '{}' into '{}'", source.display(), target.display());
            let mut g = Sodg::load(target)?;
            g.add(0)?;
            g.merge(&Sodg::load(source)?, 0, 0)?;
            let size = g.save(target)?;
            info!("The SODG saved to '{}' ({size} bytes)", target.display());
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
            info!("Dataization result, in {:?} is: {ret}", start.elapsed());
            println!("{ret}");
        }
        Some(("dot", subs)) => {
            let bin = subs
                .get_one::<PathBuf>("bin")
                .context("Path of .reo file is required")
                .unwrap();
            debug!("bin: {}", bin.display());
            if !bin.exists() {
                return Err(anyhow!("The file '{}' doesn't exist", bin.display()));
            }
            let dot = subs
                .get_one::<PathBuf>("dot")
                .context("Path of .dot file is required")
                .unwrap();
            debug!("dot: {}", dot.display());
            info!("Deserializing the binary file '{}'", bin.display());
            let g = Sodg::load(bin.as_path())?;
            info!(
                "Deserialized {} bytes in {:?}",
                fs::metadata(bin)?.len(),
                start.elapsed()
            );
            info!("Printing to '{}' file...", dot.display());
            fs::write(dot, g.to_dot())?;
            info!("File saved, in {:?}", start.elapsed());
        }
        Some(("inspect", subs)) => {
            let bin = subs
                .get_one::<PathBuf>("bin")
                .context("Path of .reo file is required")
                .unwrap();
            if !bin.exists() {
                return Err(anyhow!("The file '{}' doesn't exist", bin.display()));
            }
            println!("File: {}", bin.display());
            println!("Size: {} bytes", fs::metadata(bin)?.len());
            let g = Sodg::load(bin.as_path())?;
            println!("Total vertices: {}", g.len());
            let root = subs.get_one::<String>("root").unwrap().parse().unwrap();
            println!("\nν{root}");
            let mut seen = HashSet::new();
            seen.insert(root);
            inspect_v(&g, root, 1, &mut seen);
            println!("Vertices just printed: {}", seen.len());
            if seen.len() != g.ids().len() {
                let mut missed = vec![];
                for v in g.ids() {
                    if seen.contains(&v) {
                        continue;
                    }
                    missed.push(v);
                }
                missed.sort();
                println!(
                    "Missed: {}",
                    missed.iter().map(|v| format!("ν{}", v)).join(", ")
                );
                println!("Here they are:");
                for v in missed {
                    seen.insert(v);
                    println!("  ν{}", v);
                    inspect_v(&g, v, 2, &mut seen);
                }
            }
        }
        Some((cmd, _)) => {
            return Err(anyhow!("Can't understand '{cmd}' command"));
        }
        None => unreachable!(),
    }
    Ok(())
}

fn inspect_v(g: &Sodg, v: u32, indent: usize, seen: &mut HashSet<u32>) {
    for e in g.kids(v).unwrap() {
        print!("{}", "  ".repeat(indent));
        println!("{} -> ν{}", e.0, e.1);
        if seen.contains(&e.1) {
            continue;
        }
        seen.insert(e.1);
        inspect_v(g, e.1, indent + 1, seen);
    }
}
