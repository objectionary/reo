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
use clap::parser::ValuesRef;
use clap::ErrorKind::EmptyValue;
use clap::{crate_version, value_parser, AppSettings, Arg, ArgAction, Command};
use colored::Colorize;
use itertools::Itertools;
use log::{debug, info, warn, LevelFilter};
use reo::org::eolang::register;
use reo::Universe;
use simple_logger::SimpleLogger;
use sodg::Script;
use sodg::Sodg;
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;
use std::{fs, io};

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
                .short('v')
                .required(false)
                .help("Print all debug messages")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("trace")
                .long("trace")
                .short('t')
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
                )
                .arg_required_else_help(true),
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
                )
                .arg_required_else_help(true),
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
                        .short('r')
                        .required(false)
                        .default_value("0")
                        .help("The ID of the root vertex")
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("ignore")
                        .long("ignore")
                        .short('i')
                        .required(false)
                        .help("The IDs to ignore")
                        .value_parser(value_parser!(u32))
                        .multiple(true)
                        .action(ArgAction::Append),
                )
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("dataize")
                .setting(AppSettings::ColorNever)
                .about("Dataize an object in .reo file")
                .arg(
                    Arg::new("dump")
                        .long("dump")
                        .short('d')
                        .required(false)
                        .value_parser(PathValueParser {})
                        .help("Dump the entire graph to a file, when dataization is finished")
                        .action(ArgAction::Set),
                )
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
                    Arg::new("root")
                        .long("root")
                        .short('r')
                        .required(false)
                        .default_value("0")
                        .help("The ID of the root vertex to print")
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("ignore")
                        .long("ignore")
                        .short('i')
                        .required(false)
                        .help("The IDs to ignore")
                        .value_parser(value_parser!(u32))
                        .multiple(true)
                        .action(ArgAction::Append),
                )
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
                        .required(false)
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
            print_metas(&mut g)?;
        }
        Some(("empty", subs)) => {
            let bin = subs
                .get_one::<PathBuf>("target")
                .context("Path of .reo file is required")
                .unwrap();
            debug!("target: {}", bin.display());
            let mut g = Sodg::empty();
            g.add(0)?;
            let size = g.save(bin)?;
            info!("Empty SODG saved to '{}' ({size} bytes)", bin.display());
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
            info!("Merging into '{}':", target.display());
            let mut g1 = Sodg::load(target)?;
            print_metas(&mut g1)?;
            info!("Merging from '{}':", source.display());
            let mut g2 = Sodg::load(source)?;
            print_metas(&mut g2)?;
            let slice = g2.slice_some("ν0", |_, _, a| !a.starts_with('+'))?;
            debug!("merging {} vertices...", slice.len());
            g1.merge(&slice, 0, 0)?;
            let size = g1.save(target)?;
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
            info!("Dataizing the '{object}' object...");
            let mut uni = Universe::from_graph(g);
            register(&mut uni);
            let r = uni.dataize(format!("Φ.{}", object).as_str());
            if subs.is_present("dump") {
                let dump = subs.get_one::<PathBuf>("dump").unwrap();
                debug!("dump: {}", dump.display());
                let size = uni.dump(dump)?;
                info!("Dump saved to '{}' ({size} bytes)", dump.display());
            }
            let ret = r?.print();
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
            info!("Deserializing the binary file '{}'", bin.display());
            let g = Sodg::load(bin.as_path())?;
            info!(
                "Deserialized {} bytes in {:?}",
                fs::metadata(bin)?.len(),
                start.elapsed()
            );
            let root: u32 = subs.get_one::<String>("root").unwrap().parse().unwrap();
            let ignore: HashSet<u32> = HashSet::from_iter(
                subs.get_many("ignore")
                    .unwrap_or(ValuesRef::default())
                    .cloned(),
            );
            let content = g
                .slice_some(format!("ν{root}").as_str(), |_, v, _| !ignore.contains(&v))?
                .to_dot();
            let mut out = match subs.get_one::<PathBuf>("dot") {
                Some(f) => {
                    let path = Path::new(f);
                    info!("Printing to '{}' file...", path.display());
                    Box::new(File::create(path).unwrap()) as Box<dyn Write>
                }
                None => Box::new(io::stdout()) as Box<dyn Write>,
            };
            let bytes = out.write(content.as_bytes())?;
            info!("DOT graph saved, {bytes} bytes in {:?}", start.elapsed());
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
            let mut g = Sodg::load(bin.as_path())?;
            println!("Total vertices: {}", g.len());
            println!("Metas:");
            print_metas(&mut g)?;
            let root = subs.get_one::<String>("root").unwrap().parse().unwrap();
            let mut seen = HashSet::new();
            let ignore: Vec<u32> = subs
                .get_many("ignore")
                .unwrap_or(ValuesRef::default())
                .copied()
                .collect();
            if !ignore.is_empty() {
                println!(
                    "Ignoring: {}",
                    ignore.iter().map(|v| format!("ν{}", v)).join(", ")
                );
            }
            for v in ignore {
                seen.insert(v);
            }
            println!("\nν{root}");
            seen.insert(root);
            inspect_v(&mut g, root, 1, &mut seen);
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
                    "Missed {}: {}{}",
                    missed.len(),
                    missed.iter().take(10).map(|v| format!("ν{}", v)).join(", "),
                    if missed.len() > 10 { ", ..." } else { "" }
                );
                if missed.len() < 10 {
                    println!("Here they are:");
                    for v in missed.into_iter().take(10) {
                        seen.insert(v);
                        println!("  ν{}", v);
                        inspect_v(&mut g, v, 2, &mut seen);
                    }
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

fn print_metas(g: &mut Sodg) -> Result<()> {
    match g.kids(0) {
        Ok(vec) => {
            for (a, v) in vec {
                if a.starts_with('+') {
                    println!("  {a}: {}", g.data(v)?.to_utf8()?)
                }
            }
        }
        Err(e) => {
            warn!("  {}", e)
        }
    }
    Ok(())
}

fn inspect_v(g: &mut Sodg, v: u32, indent: usize, seen: &mut HashSet<u32>) {
    let mut kids = g.kids(v).unwrap();
    kids.sort_by(|a, b| a.0.cmp(&b.0.clone()));
    for e in kids {
        print!("{}", "  ".repeat(indent));
        print!("{} -> ν{}", e.0, e.1);
        if e.0 == "Δ" {
            print!(" {}", g.data(e.1).unwrap().to_string().blue());
        }
        if e.0 == "λ" {
            print!(" {}", g.data(e.1).unwrap().to_utf8().unwrap().yellow());
        }
        println!();
        if seen.contains(&e.1) {
            continue;
        }
        seen.insert(e.1);
        inspect_v(g, e.1, indent + 1, seen);
    }
}
