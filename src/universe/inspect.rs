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
use anyhow::{anyhow, Context, Result};
use std::collections::HashSet;

impl Universe {
    /// Finds an object by the provided locator and returns its tree
    /// of sub-objects and edges. There is no dataization happening
    /// here, just a simple traversing through attributes.
    pub fn inspect(&self, loc: &str) -> Result<String> {
        let v = self
            .find_v(loc)
            .context(format!("Can't locate '{}'", loc))?;
        let mut seen = HashSet::new();
        Ok(format!(
            "{}\n{}",
            loc,
            self.inspect_v(v, &mut seen)?.join("\n")
        ))
    }

    fn find_v(&self, loc: &str) -> Result<u32> {
        let mut v = 0;
        for k in loc.split('.') {
            if k == "Q" {
                v = 0;
                continue;
            }
            if let Some(to) = self.edge(v, k) {
                v = to;
            } else {
                return Err(anyhow!("Can't find '{}' from ν{}", k, v));
            }
        }
        Ok(v)
    }

    fn inspect_v(&self, v: u32, seen: &mut HashSet<u32>) -> Result<Vec<String>> {
        seen.insert(v);
        let mut lines = vec![];
        self.edges
            .iter()
            .filter(|(_, e)| e.from == v)
            .for_each(|(_, e)| {
                let to = self.vertices.get(&e.to).unwrap().clone();
                let line = format!(
                    "  .{} ➞ ν{}{}{}",
                    e.a,
                    e.to,
                    if to.lambda.is_some() {
                        format!(" λ{}", to.lambda_name)
                    } else {
                        "".to_string()
                    },
                    if to.data.is_some() {
                        format!(" Δ{}", to.data.unwrap().as_hex())
                    } else {
                        "".to_string()
                    }
                );
                lines.push(line);
                if !seen.contains(&e.to) && e.a != "ρ" && e.a != "𝜎" {
                    seen.insert(e.to);
                    self.inspect_v(e.to, seen)
                        .unwrap()
                        .iter()
                        .for_each(|t| lines.push(format!("  {}", t)));
                }
            });
        Ok(lines)
    }
}

#[cfg(test)]
use crate::data::Data;

#[test]
fn inspects_simple_object() -> Result<()> {
    let mut uni = Universe::empty();
    uni.add(0)?;
    uni.data(0, Data::from_int(0))?;
    uni.add(1)?;
    uni.bind(1, 0, 1, "foo")?;
    uni.atom(1, "S/Q")?;
    let txt = uni.inspect("Q")?;
    println!("{}", txt);
    assert_ne!("".to_string(), txt);
    Ok(())
}
