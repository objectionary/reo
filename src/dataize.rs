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

use anyhow::{anyhow, Context, Result};
use log::trace;
use std::collections::VecDeque;
use std::str::FromStr;
use sodg::Hex;
use crate::Universe;

impl Universe {
    /// Dataize by absolute locator. The search always starts from the
    /// root node of the tree. It is recommended to start the locator
    /// from "Φ". If you need to find any vertex starting from non-root
    /// one, use `find` method.
    pub fn dataize(&mut self, loc: &str) -> Result<Hex> {
        if self.g.is_empty() {
            return Err(anyhow!("The Universe is empty, can't dataize {}", loc));
        }
        let v = self
            .find(0, loc)
            .context(format!("Failed to find {}", loc))?;
        let data = self.g.data(v)
            .context(format!("There is no data in ν{}", v))?
            .tail(1);
        trace!(
            "#dataize: data found in ν{} ({} bytes), all good!",
            v,
            data.len()
        );
        Ok(data)
    }

    /// Find a vertex in the Universe by its locator. The search
    /// starts from the vertex `v`, but the locator may jump to
    /// the root vertex, if it starts with "Φ".
    pub fn find(&mut self, v1: u32, loc: &str) -> Result<u32> {
        trace!("#find(ν{}, '{}'): starting...", v1, loc);
        let mut v = v1;
        let mut xi = v;
        let mut xis = VecDeque::new();
        let mut locator: VecDeque<String> = VecDeque::new();
        loc.split('.')
            .for_each(|k| locator.push_back(k.to_string()));
        let mut jumps = 0;
        loop {
            let next = locator.pop_front();
            if next.is_none() {
                trace!("#find: end of locator, we are at ν{}", v);
                break;
            }
            let k = next.unwrap().to_string();
            if k.is_empty() {
                return Err(anyhow!("System error, the locator is empty"));
            }
            jumps += 1;
            if jumps > 200 {
                return Err(anyhow!(
                    "Too many jumps ({}), locator length is {}: '{}'",
                    jumps,
                    locator.len(),
                    itertools::join(locator.clone(), ".")
                ));
            }
            if k == "Δ" && self.has_data(v) {
                trace!("#find: ν{}.Δ is found!", v);
                break;
            }
            if k == "▲" {
                xi = xis.pop_back().unwrap();
                trace!("#find: ξ loaded to ν{} by ▲", xi);
                continue;
            }
            if k == "▼" {
                xis.push_back(xi);
                trace!("#find: ξ=ν{} saved by ▼", xi);
                continue;
            }
            if k.starts_with("ν") {
                let num: String = k.chars().skip(1).collect::<Vec<_>>().into_iter().collect();
                v = u32::from_str(num.as_str())?;
                xi = v;
                trace!("#find: jumping directly to ν{}", v);
                continue;
            }
            if k == "ξ" {
                v = v;
                trace!("#find: ν{}.ξ -> {}", v, v);
                continue;
            }
            if k == "Φ" || k == "Q" {
                v = 0;
                xi = v;
                trace!("#find: Φ/ν{}", v);
                continue;
            }
            if let Some(to) = self.g.kid(v, k.as_str()) {
                trace!("#find: ν{}.{} -> ν{}", v, k, to);
                v = to;
                xi = v;
                continue;
            };
            if let Some(to) = self.g.kid(v, "π") {
                trace!("#find: ν{}.π -> ν{} (.{} not found)", v, to, k);
                v = to;
                locator.push_front(k);
                continue;
            }
            if let Some(to) = self.g.kid(v, "φ") {
                trace!("#find: ν{}.φ -> ν{} (.{} not found)", v, to, k);
                v = to;
                xi = v;
                locator.push_front(k);
                continue;
            }
            if self.has_lambda(v) {
                let lname = self.lambda(v);
                trace!("#find: at ν{} calling λ{}(ξ=ν{})...", v, lname, xi);
                // let to = vtx.lambda.unwrap()(self, xi)?;
                let to = 0;
                locator.push_front(format!("ν{}", to));
                trace!("#find: λ{} in ν{}(ξ=ν{}) returned ν{}", lname, v, xi, to);
                trace!(
                    "#find: λ at λ{} reset locator to '{}'",
                    v,
                    itertools::join(locator.clone(), ".")
                );
                continue;
            }
            let others : Vec<String> = self.g.kids(v)?.into_iter().map(|(a, to)| a).collect();
            return Err(anyhow!(
                "Can't find .{} in ν{} among other {} attribute{}: {}",
                k,
                v,
                others.len(),
                if others.len() == 1 { "" } else { "s" },
                others.join(", ")
            ));
        }
        trace!("#find: found ν{} by '{}'", v1, loc);
        Ok(v)
    }
}
