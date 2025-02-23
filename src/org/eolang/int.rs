// SPDX-FileCopyrightText: Copyright (c) 2022-2025 Yegor Bugayenko
// SPDX-License-Identifier: MIT

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
