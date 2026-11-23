// SPDX-FileCopyrightText: Copyright (c) 2022-2026 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use crate::Universe;
use anyhow::Result;
use sodg::Hex;

/// Makes a fresh closed formation with the integer in its Δ-asset.
pub fn data_of_int(uni: &mut Universe, data: i64) -> Result<usize> {
    let c = uni.formation(&[])?;
    uni.delta(c, &Hex::from(data))?;
    Ok(c)
}
