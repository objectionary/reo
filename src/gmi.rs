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
        lazy_static! {
            static ref RE: Regex = Regex::new(
                "^([A-Z]+)\\((?:(?: *, *)?\"([^\"]+)\")*\\);.*$"
            ).unwrap();
        }
        let txt = &self.text.clone();
        for (pos, t) in txt.split("\n").enumerate() {
            self.deploy_one(t, uni).context(
                format!("Failure at the line no.{}: \"{}\"", pos, t)
            )?;
        }
        Ok(())
    }

    fn deploy_one(&mut self, line: &str, uni: &mut Universe) -> Result<()> {
        lazy_static! {
            static ref LINE: Regex = Regex::new(
                "^([A-Z]+)\\(((?:(?: *, *)?\"(?:[^\"]+)\")*\\)); *(?:#.*)$"
            ).unwrap();
            static ref ARGS: Regex = Regex::new(
                "(?: *, *)?\"([^\"]+)\""
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
                println!("k: {}", &k);
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
                let d : &str = &args[1];
                let bytes : Vec<u8> = (0..d.len())
                    .step_by(2)
                    .map(|i| u8::from_str_radix(&d[i..i + 2], 16).unwrap())
                    .collect();
                uni.data(v, Data::from_bytes(bytes));
            },
            _cmd => {
                return Err(anyhow!("Unknown GMI: {}", _cmd))
            }
        }
        Ok(())
    }

    /// Parses `v2` or `e5` into 2 and 5.
    fn parse(&mut self, s: &str, uni: &mut Universe) -> Result<u32> {
        let tail = &s[1..];
        if &s[0..1] == "$" {
            Ok(*self.vars.entry(tail.to_string()).or_insert(uni.next_id()))
        } else {
            Ok(u32::from_str(tail).context(format!("Parsing of \"{}\" failed", s))?)
        }
    }
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
