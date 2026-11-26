// SPDX-FileCopyrightText: Copyright (c) 2022-2026 Yegor Bugayenko
// SPDX-License-Identifier: MIT

extern crate reo;

use anyhow::Result;
use anyhow::{anyhow, Context};
use clap::builder::TypedValueParser;
use clap::ErrorKind::EmptyValue;
use clap::{crate_version, value_parser, AppSettings, Arg, ArgAction, ArgMatches, Command};
use colored::Colorize;
use itertools::Itertools;
use log::{debug, info, LevelFilter};
use reo::org::eolang::register;
use reo::{Graph, Universe};
use simple_logger::SimpleLogger;
use sodg::{Kind, Label};
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
        let path = Path::new(value);
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
                .about("Compile .sodg files, printed by `phino sodg`, into binary .reo files")
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
                        .help("The ID of the root cell")
                        .value_parser(value_parser!(u32))
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
                        .help("The ID of the root cell to print")
                        .value_parser(value_parser!(u32))
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
                        .value_parser(PathValueParser {})
                        .help("Name of a .dot file to create; prints to stdout when omitted")
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
                .context("Path of .sodg file is required")?;
            debug!("source: {}", src.display());
            if !src.exists() {
                return Err(anyhow!("The file '{}' not found", src.display()));
            }
            let bin = subs
                .get_one::<PathBuf>("target")
                .context("Path of .reo file is required")?;
            debug!("target: {}", bin.display());
            info!(
                "Compiling SODG instructions from '{}' to '{}'",
                src.display(),
                bin.display()
            );
            let g = reo::deploy(fs::read_to_string(src)?.as_str())
                .with_context(|| format!("Failed to compile '{}'", src.display()))?;
            info!("Deployed {} cells from '{}'", g.len(), src.display());
            let size = g.save(bin)?;
            info!("The SODG saved to '{}' ({size} bytes)", bin.display());
        }
        Some(("empty", subs)) => {
            let bin = subs
                .get_one::<PathBuf>("target")
                .context("Path of .reo file is required")?;
            debug!("target: {}", bin.display());
            let mut g = Graph::empty(16);
            g.formation(0, &[]);
            let size = g.save(bin)?;
            info!("Empty SODG saved to '{}' ({size} bytes)", bin.display());
        }
        Some(("merge", subs)) => {
            let target = subs
                .get_one::<PathBuf>("target")
                .context("Path of target .reo file is required")?;
            debug!("target: {}", target.display());
            if !target.exists() {
                return Err(anyhow!("The file '{}' not found", target.display()));
            }
            let source = subs
                .get_one::<PathBuf>("source")
                .context("Path of source .reo file is required")?;
            debug!("source: {}", source.display());
            if !source.exists() {
                return Err(anyhow!("The file '{}' not found", source.display()));
            }
            let g1 = Graph::load(target)?;
            info!("Merging into '{}' ({} cells)", target.display(), g1.len());
            let g2 = Graph::load(source)?;
            info!("Merging from '{}' ({} cells)", source.display(), g2.len());
            let g = reo::merge(&g1, &g2).with_context(|| {
                format!(
                    "Failed to merge '{}' into '{}'",
                    source.display(),
                    target.display()
                )
            })?;
            let size = g.save(target)?;
            info!(
                "The SODG of {} cells saved to '{}' ({size} bytes)",
                g.len(),
                target.display()
            );
        }
        Some(("dataize", subs)) => {
            let bin = subs
                .get_one::<PathBuf>("file")
                .context("Path of .reo file is required")?;
            debug!("bin: {}", bin.display());
            if !bin.exists() {
                return Err(anyhow!("The file '{}' doesn't exist", bin.display()));
            }
            let object = subs
                .get_one::<String>("object")
                .context("Object name is required")?;
            debug!("object: {object}");
            info!("Deserializing the binary file '{}'", bin.display());
            let g = Graph::load(bin.as_path())?;
            info!(
                "Deserialized {} bytes in {:?}",
                fs::metadata(bin)?.len(),
                start.elapsed()
            );
            info!("Dataizing the '{object}' object...");
            let mut uni = Universe::from_graph(g)?;
            register(&mut uni);
            let r = uni.dataize(format!("Φ.{object}").as_str());
            if let Some(dump) = subs.get_one::<PathBuf>("dump") {
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
                .context("Path of .reo file is required")?;
            debug!("bin: {}", bin.display());
            if !bin.exists() {
                return Err(anyhow!("The file '{}' doesn't exist", bin.display()));
            }
            info!("Deserializing the binary file '{}'", bin.display());
            let g = Graph::load(bin.as_path())?;
            info!(
                "Deserialized {} bytes in {:?}",
                fs::metadata(bin)?.len(),
                start.elapsed()
            );
            let root = root_of(&g, subs)?;
            let ignore: HashSet<usize> = ignored(subs).into_iter().collect();
            let content = g.slice_some(root, |_, c, _| !ignore.contains(&c))?.to_dot();
            let mut out = match subs.get_one::<PathBuf>("dot") {
                Some(f) => {
                    info!("Printing to '{}' file...", f.display());
                    Box::new(File::create(f)?) as Box<dyn Write>
                }
                None => Box::new(io::stdout()) as Box<dyn Write>,
            };
            out.write_all(content.as_bytes())?;
            info!(
                "DOT graph saved, {} bytes in {:?}",
                content.len(),
                start.elapsed()
            );
        }
        Some(("inspect", subs)) => {
            let bin = subs
                .get_one::<PathBuf>("bin")
                .context("Path of .reo file is required")?;
            if !bin.exists() {
                return Err(anyhow!("The file '{}' doesn't exist", bin.display()));
            }
            println!("File: {}", bin.display());
            println!("Size: {} bytes", fs::metadata(bin)?.len());
            let g = Graph::load(bin.as_path())?;
            let keys: HashSet<usize> = g.keys().into_iter().collect();
            println!("Total cells: {}", keys.len());
            let root = root_of(&g, subs)?;
            let ignore = ignored(subs);
            if !ignore.is_empty() {
                println!(
                    "Ignoring: {}",
                    ignore.iter().map(|c| format!("c{c}")).join(", ")
                );
            }
            let mut seen: HashSet<usize> =
                ignore.into_iter().filter(|c| keys.contains(c)).collect();
            println!("\nc{root}{}", describe(&g, root));
            seen.insert(root);
            inspect_c(&g, root, 1, &mut seen);
            println!("Cells just printed: {}", seen.len());
            let mut missed: Vec<usize> =
                keys.iter().copied().filter(|c| !seen.contains(c)).collect();
            missed.sort_unstable();
            if !missed.is_empty() {
                println!(
                    "Missed {}: {}{}",
                    missed.len(),
                    missed.iter().take(10).map(|c| format!("c{c}")).join(", "),
                    if missed.len() > 10 { ", ..." } else { "" }
                );
                if missed.len() < 10 {
                    println!("Here they are:");
                    for c in missed {
                        if !seen.insert(c) {
                            continue;
                        }
                        println!("  c{c}{}", describe(&g, c));
                        inspect_c(&g, c, 2, &mut seen);
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

/// Take the ID of the root cell from the `--root` option and make
/// sure the cell is in the graph.
fn root_of(g: &Graph, subs: &ArgMatches) -> Result<usize> {
    let root = *subs
        .get_one::<u32>("root")
        .context("The root is required")? as usize;
    if !g.keys().contains(&root) {
        return Err(anyhow!("There is no cell c{root} in the graph"));
    }
    Ok(root)
}

/// Take the IDs of the cells from the `--ignore` options.
fn ignored(subs: &ArgMatches) -> Vec<usize> {
    subs.get_many::<u32>("ignore")
        .map(|all| all.map(|c| *c as usize).collect())
        .unwrap_or_default()
}

/// Print the kind and the assets of the cell, in one line.
fn describe(g: &Graph, c: usize) -> String {
    let mut parts = vec![match g.kind(c) {
        Kind::Formation => format!("⟦{}⟧", g.attrs(c).iter().join(", ")),
        Kind::Dispatch(a) => format!("π.{a}"),
        Kind::Application(a) => format!("π({a} ↦ …)"),
        Kind::Termination => String::from("⊥"),
        Kind::Void => String::from("∅"),
    }];
    if let Some(d) = g.data(c) {
        parts.push(format!("Δ ⤍ {}", d.to_string().blue()));
    }
    if let Some(a) = g.atom(c) {
        parts.push(format!("λ ⤍ {}", a.yellow()));
    }
    format!(" {}", parts.join(" "))
}

/// Print the edges of the cell and, recursively, their kids,
/// which were not seen yet.
fn inspect_c(g: &Graph, c: usize, indent: usize, seen: &mut HashSet<usize>) {
    let kids: Vec<(Label, usize)> = g.kids(c).map(|(a, k)| (*a, *k)).sorted().collect();
    for (a, k) in kids {
        let fresh = seen.insert(k);
        println!(
            "{}.{a} ➞ c{k}{}",
            "  ".repeat(indent),
            if fresh {
                describe(g, k)
            } else {
                String::from(" …")
            }
        );
        if fresh {
            inspect_c(g, k, indent + 1, seen);
        }
    }
}
