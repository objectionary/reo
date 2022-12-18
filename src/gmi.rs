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

use crate::data::Data;
use crate::universe::Universe;
use anyhow::{anyhow, Context, Result};
use itertools::Itertools;
use lazy_static::lazy_static;
use log::trace;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;

/// Collection of GMIs, which can be deployed to a `Universe`.
pub struct Gmi {
    vars: HashMap<String, u32>,
    text: String,
}

impl Gmi {
    /// Read them from a file.
    pub fn from_file(file: &Path) -> Result<Gmi> {
        Gmi::from_string(
            fs::read_to_string(file).context(format!("Can't read from \"{}\"", file.display()))?,
        )
    }

    /// Read them from a string.
    pub fn from_string(text: String) -> Result<Gmi> {
        Ok(Gmi {
            text,
            vars: HashMap::new(),
        })
    }

    /// Set root.
    pub fn set_root(&mut self, v0: u32) {
        self.vars.insert("ν0".to_string(), v0);
    }

    /// Deploy this collection of GMIs to the Universe. Returns total
    /// number of GMI instructions deployed.
    pub fn deploy_to(&mut self, uni: &mut Universe) -> Result<u32> {
        let txt = &self.text.clone();
        let lines = txt.split('\n').map(|t| t.trim()).filter(|t| !t.is_empty());
        let mut total = 0;
        for (pos, t) in lines.enumerate() {
            if t.starts_with('#') {
                continue;
            }
            trace!("#deploy_to: deploying line no.{} '{}'...", pos + 1, t);
            self.deploy_one(t, uni)
                .context(format!("Failure at the line no.{}: '{}'", pos, t))?;
            total += 1;
        }
        Ok(total)
    }

    /// Deploy a sing command to the universe.
    fn deploy_one(&mut self, line: &str, uni: &mut Universe) -> Result<()> {
        lazy_static! {
            static ref LINE: Regex = Regex::new(
                "^([A-Z]+) *\\( *((?:(?: *, *)?(?:'|\")(?:[^'\"]+)(?:'|\"))* *\\)) *; *(?:#.*)?$"
            )
            .unwrap();
            static ref ARGS: Regex = Regex::new("(?: *, *)?(?:'|\")([^\"'']+)(?:'|\")").unwrap();
            static ref LOC: Regex = Regex::new("(^|\\.)\\$").unwrap();
        }
        let cap = LINE
            .captures(line)
            .context(format!("Can't parse '{}'", line))?;
        let args: Vec<&str> = ARGS
            .captures_iter(&cap[2])
            .map(|c| c.get(1).unwrap().as_str())
            .collect();
        match &cap[1] {
            "ADD" => {
                let v = self.parse(args[0], uni)?;
                uni.add(v).context(format!("Failed to ADD({})", &args[0]))
            }
            "BIND" => {
                let e = self.parse(args[0], uni)?;
                let v1 = self.parse(args[1], uni)?;
                let v2 = self.parse(args[2], uni)?;
                let a = &args[3];
                uni.bind(e, v1, v2, a).context(format!(
                    "Failed to BIND({}, {}, {})",
                    &args[0], &args[1], &args[2]
                ))
            }
            "COPY" => {
                let e1 = self.parse(args[0], uni)?;
                let v3 = self.parse(args[1], uni)?;
                let e2 = self.parse(args[2], uni)?;
                uni.copy(e1, v3, e2).context(format!(
                    "Failed to COPY({}, {}, {})",
                    &args[0], &args[1], &args[2]
                ))
            }
            "DATA" => {
                let v = self.parse(args[0], uni)?;
                uni.data(v, Self::parse_data(args[1])?)
                    .context(format!("Failed to DATA({})", &args[0]))
            }
            "ATOM" => {
                let v = self.parse(args[0], uni)?;
                let m = &args[1];
                uni.atom(v, m)
                    .context(format!("Failed to ATOM({})", &args[0]))
            }
            _cmd => Err(anyhow!("Unknown GMI: {}", _cmd)),
        }
    }

    /// Parse data
    fn parse_data(s: &str) -> Result<Data> {
        lazy_static! {
            static ref DATA_STRIP: Regex = Regex::new("[ \t\n\r\\-]").unwrap();
            static ref DATA: Regex = Regex::new("^[0-9A-Fa-f]{2}([0-9A-Fa-f]{2})*$").unwrap();
        }
        let d: &str = &DATA_STRIP.replace_all(s, "");
        let data = if DATA.is_match(d) {
            let bytes: Vec<u8> = (0..d.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&d[i..i + 2], 16).unwrap())
                .collect();
            Data::from_bytes(bytes)
        } else {
            let (t, tail) = d
                .splitn(2, '/')
                .collect_tuple()
                .context(format!("Strange data format: '{}'", d))?;
            match t {
                "bytes" => Data::from_hex(tail.to_string()),
                "string" => Data::from_string(Self::unescape(tail)),
                "int" => Data::from_int(i64::from_str(tail)?),
                "float" => Data::from_float(f64::from_str(tail)?),
                "bool" => Data::from_bool(tail == "true"),
                "array" => Data::from_bool(true),
                _ => return Err(anyhow!("Unknown type of data '{}'", t)),
            }
        };
        Ok(data)
    }

    /// Parses `ε2` or `ν5` into `2` and `5` respectively.
    fn parse(&mut self, s: &str, uni: &mut Universe) -> Result<u32> {
        if s == "$ν0" {
            return Err(anyhow!("It's illegal to use {}", s));
        }
        if s == "ν0" {
            return Ok(if let Some(v) = self.vars.get(s) {
                *v
            } else {
                0
            });
        }
        let head = s.chars().next().context("Empty identifier".to_string())?;
        let tail: String = s.chars().skip(1).collect::<Vec<_>>().into_iter().collect();
        if head == '$' {
            Ok(*self.vars.entry(tail.to_string()).or_insert_with(|| {
                match tail
                    .chars()
                    .next()
                    .context("Empty prefix".to_string())
                    .unwrap()
                {
                    'ν' => uni.next_v(),
                    'ε' => uni.next_e(),
                    p => panic!("Unknown prefix '{}' in {}", p, tail),
                }
            }))
        } else {
            Ok(u32::from_str(tail.as_str()).context(format!("Parsing of '{}' failed", s))?)
        }
    }

    /// Goes through the string and replaces `\u0027` (for example)
    /// with corresponding chars.
    fn unescape(s: &str) -> String {
        // todo!
        s.to_string()
    }
}

#[cfg(test)]
use crate::da;

#[test]
fn deploys_simple_commands() -> Result<()> {
    let uni: &mut Universe = &mut Universe::empty();
    Gmi::from_string(
        "
        ADD('ν0');
        ADD('$ν1');
        BIND('ε2', 'ν0', '$ν1', 'foo');
        DATA('$ν1', 'd0 bf d1 80 d0 b8 d0 b2 d0 b5 d1 82');
        "
        .to_string(),
    )?
    .deploy_to(uni)?;
    assert_eq!("привет", da!(uni, "Φ.foo").as_string()?);
    Ok(())
}

#[test]
fn repositions_root() -> Result<()> {
    let mut gmi = Gmi::from_string(
        "
        # Just a simple object with a edge from root
        ADD('$ν1');
        BIND('$ε1', 'ν0', '$ν1', 'foo');
        "
        .to_string(),
    )?;
    let uni: &mut Universe = &mut Universe::empty();
    uni.add(256)?;
    uni.add(42)?;
    gmi.set_root(42);
    gmi.deploy_to(uni)?;
    assert_eq!(1, uni.find(256, "ν42.foo")?);
    Ok(())
}
