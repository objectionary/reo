// SPDX-FileCopyrightText: Copyright (c) 2022-2026 Yegor Bugayenko
// SPDX-License-Identifier: MIT

//! Conversions between EO objects and Rust values, which atoms use
//! to read their arguments and to make their results, in the same way
//! as `Data.ToPhi`, `Numeric`, and `Int` do in eo-runtime.

use crate::Universe;
use anyhow::{bail, Context, Result};
use sodg::Hex;

/// Make `Φ.bytes(φ ↦ data)`, the way EO wraps a byte array.
pub fn bytes_of(uni: &mut Universe, data: &[u8]) -> Result<usize> {
    let proto = uni.dispatch(0, "bytes")?;
    let d = uni.formation(&[])?;
    uni.delta(d, &Hex::from_slice(data))?;
    uni.application(proto, "φ", d)
}

/// Make `Φ.number(φ ↦ Φ.bytes(φ ↦ data))`, where the data is the
/// 64-bit float in big-endian order.
pub fn number_of(uni: &mut Universe, num: f64) -> Result<usize> {
    let proto = uni.dispatch(0, "number")?;
    let b = bytes_of(uni, &num.to_be_bytes())?;
    uni.application(proto, "φ", b)
}

/// Make `Φ.string(φ ↦ Φ.bytes(φ ↦ data))`, where the data is UTF-8.
pub fn string_of(uni: &mut Universe, txt: &str) -> Result<usize> {
    let proto = uni.dispatch(0, "string")?;
    let b = bytes_of(uni, txt.as_bytes())?;
    uni.application(proto, "φ", b)
}

/// Find `Φ.true` or `Φ.false`.
pub fn bool_of(uni: &mut Universe, b: bool) -> Result<usize> {
    uni.find(if b { "Φ.true" } else { "Φ.false" })
}

/// Dataize the object as a 64-bit float, which must be exactly
/// eight bytes.
pub fn number_at(uni: &mut Universe, loc: &str) -> Result<f64> {
    let d = uni.dataize(loc)?;
    let bytes: [u8; 8] = d
        .bytes()
        .try_into()
        .with_context(|| format!("{loc} must be a number, but it is {d}"))?;
    Ok(f64::from_be_bytes(bytes))
}

/// Dataize the object as a number, which must be an integer
/// within the range of `i32`.
pub fn int_at(uni: &mut Universe, loc: &str) -> Result<i32> {
    let num = number_at(uni, loc)?;
    if num.fract() != 0.0 {
        bail!("{loc} must be an integer, but it is {num}");
    }
    if num < f64::from(i32::MIN) || num > f64::from(i32::MAX) {
        bail!("{loc} must fit into int range, but it is {num}");
    }
    #[allow(clippy::cast_possible_truncation)]
    Ok(num as i32)
}

/// Dataize the object as a number, which must be a non-negative
/// integer within the range of `i32`.
pub fn natural_at(uni: &mut Universe, loc: &str) -> Result<usize> {
    let num = int_at(uni, loc)?;
    usize::try_from(num)
        .with_context(|| format!("{loc} must be greater or equal to zero, but it is {num}"))
}
