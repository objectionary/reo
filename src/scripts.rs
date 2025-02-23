// SPDX-FileCopyrightText: Copyright (c) 2022-2025 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use crate::Universe;
use anyhow::Result;
use sodg::Hex;

/// Makes a copy of `org.eolang.int` in the Universe. It is assumed
/// that it already exists there.
pub fn copy_of_int(uni: &mut Universe, data: i64) -> Result<u32> {
    let v = uni.add();
    let int = uni.find("org.eolang.int")?;
    uni.bind(v, int, "π");
    let d = uni.add();
    uni.put(d, Hex::from(data));
    uni.bind(v, d, "Δ");
    Ok(v)
}
