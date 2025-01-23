// Copyright (c) 2022-2025 Yegor Bugayenko
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

use crate::scripts::copy_of_int;
use crate::Universe;
use anyhow::Result;

/// Register all known atoms in the Universe.
pub fn register(uni: &mut Universe) {
    uni.register("org.eolang.int$plus", int_plus);
    uni.register("org.eolang.int$times", int_times);
    uni.register("org.eolang.int$div", int_div);
}

/// EO atom `int.plus`.
pub fn int_plus(uni: &mut Universe, v: u32) -> Result<u32> {
    let rho = uni.dataize(format!("ν{}.ρ", v).as_str())?.to_i64()?;
    let x = uni.dataize(format!("ν{}.α0", v).as_str())?.to_i64()?;
    copy_of_int(uni, rho + x)
}

/// EO atom `int.times`.
pub fn int_times(uni: &mut Universe, v: u32) -> Result<u32> {
    let rho = uni.dataize(format!("ν{}.ρ", v).as_str())?.to_i64()?;
    let x = uni.dataize(format!("ν{}.α0", v).as_str())?.to_i64()?;
    copy_of_int(uni, rho * x)
}

/// EO atom `int.div`.
pub fn int_div(uni: &mut Universe, v: u32) -> Result<u32> {
    let rho = uni.dataize(format!("ν{}.ρ", v).as_str())?.to_i64()?;
    let x = uni.dataize(format!("ν{}.α0", v).as_str())?.to_i64()?;
    copy_of_int(uni, rho / x)
}

#[test]
fn simple() {
    // assert_eq!(1, total);
}
