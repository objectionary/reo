// SPDX-FileCopyrightText: Copyright (c) 2022-2026 Yegor Bugayenko
// SPDX-License-Identifier: MIT

//! The atoms of EO's `number`, which is a 64-bit float.

use crate::org::eolang::data::{bool_of, number_at, number_of};
use crate::Universe;
use anyhow::Result;

/// Register all atoms of `number`.
pub fn register(uni: &mut Universe) {
    uni.register("L_number_plus", number_plus);
    uni.register("L_number_times", number_times);
    uni.register("L_number_div", number_div);
    uni.register("L_number_gt", number_gt);
}

/// Dataize `ρ` and `b` of the atom `c` as numbers.
fn operands(uni: &mut Universe, c: usize) -> Result<(f64, f64)> {
    let rho = number_at(uni, format!("ν{c}.ρ").as_str())?;
    let b = number_at(uni, format!("ν{c}.v🌵b").as_str())?;
    Ok((rho, b))
}

/// EO atom `number.plus`.
pub fn number_plus(uni: &mut Universe, c: usize) -> Result<usize> {
    let (rho, b) = operands(uni, c)?;
    number_of(uni, rho + b)
}

/// EO atom `number.times`.
pub fn number_times(uni: &mut Universe, c: usize) -> Result<usize> {
    let (rho, b) = operands(uni, c)?;
    number_of(uni, rho * b)
}

/// EO atom `number.div`, which gives infinity or NaN when
/// dividing by zero, like Java does.
pub fn number_div(uni: &mut Universe, c: usize) -> Result<usize> {
    let (rho, b) = operands(uni, c)?;
    number_of(uni, rho / b)
}

/// EO atom `number.gt`.
pub fn number_gt(uni: &mut Universe, c: usize) -> Result<usize> {
    let (rho, b) = operands(uni, c)?;
    bool_of(uni, rho > b)
}
