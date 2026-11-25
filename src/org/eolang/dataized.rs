// SPDX-FileCopyrightText: Copyright (c) 2022-2026 Yegor Bugayenko
// SPDX-License-Identifier: MIT

//! The atom of EO's `dataized`.

use crate::org::eolang::data::bytes_of;
use crate::Universe;
use anyhow::Result;

/// Register the atom of `dataized`.
pub fn register(uni: &mut Universe) {
    uni.register("L_dataized", dataized);
}

/// EO atom `dataized`, which gives the data of its `target` as bytes.
pub fn dataized(uni: &mut Universe, c: usize) -> Result<usize> {
    let d = uni.dataize(format!("ν{c}.v🌵target").as_str())?;
    bytes_of(uni, d.bytes())
}
