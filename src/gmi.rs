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

use crate::universe::Universe;
use std::fs;
use std::collections::HashMap;
use anyhow::{anyhow, Context, Result};
use std::str::FromStr;
use lazy_static::lazy_static;
use regex::Regex;
use crate::data::Data;
use itertools::Itertools;

/// Collection of GMIs, which can be deployed to a `Universe`.
pub struct Gmi {
    vars: HashMap<String, u32>,
    text: String,
}

impl Gmi {
    /// Read them from a file.
    pub fn from_file(file: &str) -> Result<Gmi> {
        return Gmi::from_string(fs::read_to_string(file)?);
    }

    /// Read them from a string.
    pub fn from_string(text: String) -> Result<Gmi> {
        return Ok(
            Gmi {
                text,
                vars: HashMap::new()
            }
        );
    }

    pub fn deploy_to(&mut self, uni: &mut Universe) -> Result<()> {
        let txt = &self.text.clone();
        let lines = txt.split("\n").map(|t| t.trim()).filter(|t| !t.is_empty());
        for (pos, t) in lines.enumerate() {
            self.deploy_one(t, uni).context(
                format!("Failure at the line no.{}: \"{}\"", pos, t)
            )?;
        }
        Ok(())
    }

    /// Deploy a sing command to the universe.
    fn deploy_one(&mut self, line: &str, uni: &mut Universe) -> Result<()> {
        lazy_static! {
            static ref LINE: Regex = Regex::new(
                "^([A-Z]+) *\\( *((?:(?: *, *)?(?:'|\")(?:[^'\"]+)(?:'|\"))* *\\)) *; *(?:#.*)?$"
            ).unwrap();
            static ref ARGS: Regex = Regex::new(
                "(?: *, *)?(?:'|\")([^\"'']+)(?:'|\")"
            ).unwrap();
            static ref LOC: Regex = Regex::new(
                "(^|\\.)\\$"
            ).unwrap();
        }
        let cap = LINE.captures(line).context(format!("Can't parse \"{}\"", line))?;
        let args : Vec<&str> = ARGS.captures_iter(&cap[2])
            .map(|c| c.get(1).unwrap().as_str())
            .collect();
        match &cap[1] {
            "ADD" => {
                let v = self.parse(&args[0], uni)?;
                uni.add(v);
            },
            "BIND" => {
                let e = self.parse(&args[0], uni)?;
                let v1 = self.parse(&args[1], uni)?;
                let v2 = self.parse(&args[2], uni)?;
                let a = &args[3];
                uni.bind(e, v1, v2, a);
            },
            "REF" => {
                let e1 = self.parse(&args[0], uni)?;
                let v1 = self.parse(&args[1], uni)?;
                let k = LOC.replace_all(&args[2], "$1");
                let a = &args[3];
                uni.reff(e1, v1, &k, a);
            },
            "COPY" => {
                let e1 = self.parse(&args[0], uni)?;
                let v3 = self.parse(&args[1], uni)?;
                let e2 = self.parse(&args[2], uni)?;
                uni.copy(e1, v3, e2);
            },
            "DATA" => {
                let v = self.parse(&args[0], uni)?;
                uni.data(v, Self::parse_data(&args[1])?);
            },
            _cmd => {
                return Err(anyhow!("Unknown GMI: {}", _cmd))
            }
        }
        Ok(())
    }

    /// Parse data
    fn parse_data(s: &str) -> Result<Data> {
        lazy_static! {
            static ref DATA_STRIP: Regex = Regex::new(
                "[^0-9A-Fa-f]"
            ).unwrap();
            static ref DATA: Regex = Regex::new(
                "^[0-9A-Fa-f]{2}([0-9A-Fa-f]{2})*$"
            ).unwrap();
        }
        let d : &str = &DATA_STRIP.replace_all(s, "");
        let data = if DATA.is_match(d) {
            let bytes : Vec<u8> = (0..d.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&d[i..i + 2], 16).unwrap())
                .collect();
            Data::from_bytes(bytes)
        } else {
            let (t, tail) = d.splitn(2, "/")
                .collect_tuple()
                .context(format!("Strange data format: \"{}\"", d))?;
            match t {
                "string" => Data::from_string(tail.to_string()),
                "int" => Data::from_int(i64::from_str(tail)?),
                "float" => Data::from_float(f64::from_str(tail)?),
                "bool" => Data::from_bool(tail == "true"),
                _ => {
                    return Err(anyhow!("Unknown type of data \"{}\"", t))
                }
            }
        };
        Ok(data)
    }

    /// Parses `v2` or `e5` into 2 and 5.
    fn parse(&mut self, s: &str, uni: &mut Universe) -> Result<u32> {
        let tail : String = s.chars().skip(1).collect::<Vec<_>>()
            .into_iter().collect();
        if s.chars().next().unwrap() == '$' {
            Ok(*self.vars.entry(tail.to_string()).or_insert(uni.next_id()))
        } else {
            Ok(u32::from_str(tail.as_str()).context(format!("Parsing of \"{}\" failed", s))?)
        }
    }
}

#[test]
fn deploys_simple_commands() {
    let uni : &mut Universe = &mut Universe::empty();
    uni.add(0);
    Gmi::from_string(
        "
        ADD('ν1'); # just first vertex
        BIND('ε1', 'ν0', 'ν1', 'foo'); # just first vertex
        ADD('ν2'); # its data vertex
        BIND ( 'ε2', 'ν1', 'ν2', 'Δ' );
        DATA('ν2', 'd0 bf d1 80 d0 b8 d0 b2 d0 b5 d1 82');
        ".to_string()
    ).unwrap().deploy_to(uni).unwrap();
    assert_eq!("привет", uni.dataize(0, "foo.Δ").unwrap().as_string());
}

#[test]
fn deploys_fibonacci() -> Result<()> {
    let uni : &mut Universe = &mut Universe::empty();
    uni.add(0);
    Gmi::from_file("target/eo/gmi/org/eolang/reo/fibonacci.gmi")?
        .deploy_to(uni)?;
    assert_eq!(8, uni.dataize(0, "fibonacci.f").unwrap().as_int());
    Ok(())
}
