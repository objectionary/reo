// SPDX-FileCopyrightText: Copyright (c) 2022-2026 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use crate::scripts::copy_of_int;
use crate::Universe;
use anyhow::{anyhow, Result};

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
    if x == 0 {
        return Err(anyhow!("Can't divide {rho} by zero"));
    }
    copy_of_int(uni, rho / x)
}

#[cfg(test)]
use sodg::Hex;

/// Build `org.eolang.int` reachable from the root, so that
/// [`copy_of_int`] can find it.
#[cfg(test)]
fn make_int_object(uni: &mut Universe) {
    let root = uni.add();
    assert_eq!(0, root);
    let org = uni.add();
    uni.bind(root, org, "org");
    let eolang = uni.add();
    uni.bind(org, eolang, "eolang");
    let int = uni.add();
    uni.bind(eolang, int, "int");
}

/// Make a vertex that emulates an `int.div`/`int.plus`-style call,
/// with `ρ` (the left operand) and `α0` (the right operand) attached.
#[cfg(test)]
fn make_call(uni: &mut Universe, rho: i64, x: i64) -> u32 {
    let v = uni.add();
    let rho_v = uni.add();
    uni.bind(v, rho_v, "ρ");
    let rho_d = uni.add();
    uni.bind(rho_v, rho_d, "Δ");
    uni.put(rho_d, Hex::from(rho));
    let x_v = uni.add();
    uni.bind(v, x_v, "α0");
    let x_d = uni.add();
    uni.bind(x_v, x_d, "Δ");
    uni.put(x_d, Hex::from(x));
    v
}

#[test]
fn divides_two_integers() -> Result<()> {
    let mut uni = Universe::empty();
    make_int_object(&mut uni);
    let v = make_call(&mut uni, 12, 4);
    let result = int_div(&mut uni, v)?;
    assert_eq!(3, uni.dataize(format!("ν{result}").as_str())?.to_i64()?);
    Ok(())
}

#[test]
fn fails_to_divide_by_zero() {
    let mut uni = Universe::empty();
    make_int_object(&mut uni);
    let v = make_call(&mut uni, 42, 0);
    assert!(
        int_div(&mut uni, v).is_err(),
        "Division by zero must return an error, not panic"
    );
}
