// SPDX-FileCopyrightText: Copyright (c) 2022-2026 Yegor Bugayenko
// SPDX-License-Identifier: MIT

//! The atoms of EO's `bytes`.

use crate::org::eolang::data::{bool_of, bytes_of, int_at, natural_at, number_of, string_of};
use crate::Universe;
use anyhow::{bail, Result};

/// Register all atoms of `bytes`.
pub fn register(uni: &mut Universe) {
    uni.register("L_bytes_and", bytes_and);
    uni.register("L_bytes_or", bytes_or);
    uni.register("L_bytes_not", bytes_not);
    uni.register("L_bytes_concat", bytes_concat);
    uni.register("L_bytes_eq", bytes_eq);
    uni.register("L_bytes_left", bytes_left);
    uni.register("L_bytes_right", bytes_right);
    uni.register("L_bytes_size", bytes_size);
    uni.register("L_bytes_slice", bytes_slice);
}

/// Dataize `ρ` of the atom `c`.
fn rho(uni: &mut Universe, c: usize) -> Result<Vec<u8>> {
    Ok(uni.dataize(format!("ν{c}.ρ").as_str())?.bytes().to_vec())
}

/// Dataize `b` of the atom `c`.
fn arg(uni: &mut Universe, c: usize) -> Result<Vec<u8>> {
    Ok(uni.dataize(format!("ν{c}.v🌵b").as_str())?.bytes().to_vec())
}

/// Apply the operation to each pair of bytes of `ρ` and `b`, which
/// must be of the same length.
fn bitwise(uni: &mut Universe, c: usize, op: &str, f: fn(u8, u8) -> u8) -> Result<usize> {
    let left = rho(uni, c)?;
    let right = arg(uni, c)?;
    if left.len() != right.len() {
        bail!(
            "bytes.{op} requires operands of the same length, got {} and {}",
            left.len(),
            right.len()
        );
    }
    let out: Vec<u8> = left.iter().zip(&right).map(|(a, b)| f(*a, *b)).collect();
    bytes_of(uni, &out)
}

/// Shift the bits of the bytes, as one big-endian number of the same
/// size: to the right if `bits` is positive, to the left otherwise,
/// filling the vacant bits with zeros, like `BytesRaw.shift` in eo-runtime.
fn shifted(bytes: &[u8], bits: i64) -> Vec<u8> {
    let total = bytes.len() * 8;
    let mut out = vec![0_u8; bytes.len()];
    for k in 0..total {
        let src = i64::try_from(k).map(|k| k - bits).ok();
        let Some(src) = src.and_then(|s| usize::try_from(s).ok()) else {
            continue;
        };
        if src < total && bytes[src / 8] & (0x80 >> (src % 8)) != 0 {
            out[k / 8] |= 0x80 >> (k % 8);
        }
    }
    out
}

/// EO atom `bytes.and`.
pub fn bytes_and(uni: &mut Universe, c: usize) -> Result<usize> {
    bitwise(uni, c, "and", |a, b| a & b)
}

/// EO atom `bytes.or`.
pub fn bytes_or(uni: &mut Universe, c: usize) -> Result<usize> {
    bitwise(uni, c, "or", |a, b| a | b)
}

/// EO atom `bytes.not`.
pub fn bytes_not(uni: &mut Universe, c: usize) -> Result<usize> {
    let out: Vec<u8> = rho(uni, c)?.iter().map(|b| !b).collect();
    bytes_of(uni, &out)
}

/// EO atom `bytes.concat`.
pub fn bytes_concat(uni: &mut Universe, c: usize) -> Result<usize> {
    let mut out = rho(uni, c)?;
    out.extend(arg(uni, c)?);
    bytes_of(uni, &out)
}

/// EO atom `bytes.eq`, which compares `ρ` with `b.as-bytes`.
pub fn bytes_eq(uni: &mut Universe, c: usize) -> Result<usize> {
    let other = uni.dataize(format!("ν{c}.v🌵b.as-bytes").as_str())?;
    let me = rho(uni, c)?;
    bool_of(uni, me == other.bytes())
}

/// EO atom `bytes.left`, which shifts the bits to the left by `b`.
pub fn bytes_left(uni: &mut Universe, c: usize) -> Result<usize> {
    let bits = int_at(uni, format!("ν{c}.v🌵b").as_str())?;
    let out = shifted(&rho(uni, c)?, -i64::from(bits));
    bytes_of(uni, &out)
}

/// EO atom `bytes.right`, which shifts the bits to the right by `b`.
pub fn bytes_right(uni: &mut Universe, c: usize) -> Result<usize> {
    let bits = int_at(uni, format!("ν{c}.v🌵b").as_str())?;
    let out = shifted(&rho(uni, c)?, i64::from(bits));
    bytes_of(uni, &out)
}

/// EO atom `bytes.size`.
pub fn bytes_size(uni: &mut Universe, c: usize) -> Result<usize> {
    let len = rho(uni, c)?.len();
    #[allow(clippy::cast_precision_loss)]
    number_of(uni, len as f64)
}

/// EO atom `bytes.slice`, which takes `len` bytes from `start`, or,
/// if they are out of bounds, gives `cant-slice` with the message
/// in its first void attribute.
pub fn bytes_slice(uni: &mut Universe, c: usize) -> Result<usize> {
    let bytes = rho(uni, c)?;
    let start = natural_at(uni, format!("ν{c}.v🌵start").as_str())?;
    let len = natural_at(uni, format!("ν{c}.v🌵len").as_str())?;
    if start + len <= bytes.len() {
        return bytes_of(uni, &bytes[start..start + len]);
    }
    let msg = format!(
        "cannot slice '{len}' bytes from offset '{start}' of bytes of size {}",
        bytes.len()
    );
    let fallback = format!("ν{c}.v🌵cant-slice");
    let void = uni.first_void(fallback.as_str())?;
    let proto = uni.find(fallback.as_str())?;
    let txt = string_of(uni, msg.as_str())?;
    uni.application(proto, void.as_str(), txt)
}

#[cfg(test)]
mod tests {
    use super::shifted;

    #[test]
    fn shifts_like_eo() {
        assert_eq!(vec![0x00, 0x01], shifted(&[0x01, 0x00], 8));
        assert_eq!(vec![0x02, 0x00], shifted(&[0x01, 0x00], -1));
        assert_eq!(vec![0x00, 0x80], shifted(&[0x01, 0x00], 1));
        assert_eq!(vec![0x01, 0x00], shifted(&[0x00, 0x80], -1));
        assert_eq!(vec![0x00, 0x00], shifted(&[0xFF, 0xFF], 16));
        assert_eq!(vec![0x3F, 0xFF], shifted(&[0xFF, 0xFF], 2));
    }
}
