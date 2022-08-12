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
use std::{fs, io};
use std::str::FromStr;
use lazy_static::lazy_static;
use regex::Regex;
use crate::data::Data;

// Collection of GMIs, which can be deployed to a `Universe`.
pub struct Gmi {
    text: String,
}

impl Gmi {
    // Read them from a file.
    pub fn from_file(file: &str) -> io::Result<Gmi> {
        return Gmi::from_string(fs::read_to_string(file)?);
    }

    // Read them from a string.
    pub fn from_string(text: String) -> io::Result<Gmi> {
        return Ok(
            Gmi {
                text
            }
        );
    }

    pub fn deploy_to(&self, uni: &mut Universe) {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                "^([A-Z]+)\\((?:(?: *, *)?\"([^\"]+)\")*\\);.*$"
            ).unwrap();
        }
        self.text.split("\n").for_each(|t| {
            if let Some(cap) = RE.captures(t) {
                println!("cmd: {} {}", &cap[1], &cap[2]);
                match &cap[1] {
                    "ADD" => {
                        let v = Self::parse(&cap[2]);
                        uni.add(v);
                    },
                    "BIND" => {
                        let e = Self::parse(&cap[2]);
                        let v1 = Self::parse(&cap[3]);
                        let v2 = Self::parse(&cap[4]);
                        let a = &cap[5];
                        uni.bind(e, v1, v2, a);
                    },
                    "REF" => {
                        let e1 = Self::parse(&cap[2]);
                        let v1 = Self::parse(&cap[3]);
                        let l = &cap[4];
                        let a = &cap[5];
                        uni.reff(e1, v1, l, a);
                    },
                    "COPY" => {
                        let e1 = Self::parse(&cap[2]);
                        let v3 = Self::parse(&cap[3]);
                        let e2 = Self::parse(&cap[4]);
                        uni.copy(e1, v3, e2);
                    },
                    "DATA" => {
                        let v = Self::parse(&cap[2]);
                        let d : &str = &cap[3];
                        let bytes : Vec<u8> = (0..d.len())
                            .step_by(2)
                            .map(|i| u8::from_str_radix(&d[i..i + 2], 16).unwrap())
                            .collect();
                        uni.data(v, Data::from_bytes(bytes));
                    },
                    _ => {
                        panic!("Unknown GMI: {}", &cap[1])
                    }
                };
            } else {
                panic!("Can't parse GMI line: \"{}\"", t);
            }
        });
    }

    // Parses `v2` or `e5` into 2 and 5.
    fn parse(s: &str) -> u32{
        let tail = &s[1..];
        u32::from_str(tail).unwrap()
    }
}

#[test]
fn deploys_fibonacci() {
    let uni : &mut Universe = &mut Universe::empty();
    Gmi::from_file("target/eo/gmi/org/eolang/reo/fibonacci.gmi")
        .unwrap()
        .deploy_to(uni);
    assert_eq!(8, uni.dataize(0, "fibonacci.f").unwrap().as_int());
}
