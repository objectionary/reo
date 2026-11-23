// SPDX-FileCopyrightText: Copyright (c) 2022-2026 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use crate::scripts::data_of_int;
use crate::Universe;
use anyhow::{anyhow, Result};

/// Register all known atoms in the Universe.
pub fn register(uni: &mut Universe) {
    uni.register("L_int_plus", int_plus);
    uni.register("L_int_times", int_times);
    uni.register("L_int_div", int_div);
}

/// EO atom `int.plus`.
pub fn int_plus(uni: &mut Universe, c: usize) -> Result<usize> {
    let rho = uni.dataize(format!("ν{c}.ρ").as_str())?.to_i64()?;
    let x = uni.dataize(format!("ν{c}.x").as_str())?.to_i64()?;
    let sum = rho
        .checked_add(x)
        .ok_or_else(|| anyhow!("Integer overflow in {rho} + {x}"))?;
    data_of_int(uni, sum)
}

/// EO atom `int.times`.
pub fn int_times(uni: &mut Universe, c: usize) -> Result<usize> {
    let rho = uni.dataize(format!("ν{c}.ρ").as_str())?.to_i64()?;
    let x = uni.dataize(format!("ν{c}.x").as_str())?.to_i64()?;
    let product = rho
        .checked_mul(x)
        .ok_or_else(|| anyhow!("Integer overflow in {rho} * {x}"))?;
    data_of_int(uni, product)
}

/// EO atom `int.div`.
pub fn int_div(uni: &mut Universe, c: usize) -> Result<usize> {
    let rho = uni.dataize(format!("ν{c}.ρ").as_str())?.to_i64()?;
    let x = uni.dataize(format!("ν{c}.x").as_str())?.to_i64()?;
    if x == 0 {
        return Err(anyhow!("Can't divide {rho} by zero"));
    }
    let quotient = rho
        .checked_div(x)
        .ok_or_else(|| anyhow!("Integer overflow in {rho} / {x}"))?;
    data_of_int(uni, quotient)
}

#[cfg(test)]
use sodg::Hex;

/// Make a formation that emulates an `int.div`/`int.plus`-style call,
/// with `ρ` (the left operand) and `x` (the right operand) attached.
#[cfg(test)]
fn make_call(uni: &mut Universe, rho: i64, x: i64) -> Result<usize> {
    let root = uni.formation(&[])?;
    assert_eq!(0, root);
    let c = uni.formation(&["ρ", "x"])?;
    let left = uni.formation(&[])?;
    uni.delta(left, &Hex::from(rho))?;
    uni.put(c, "ρ", left)?;
    let right = uni.formation(&[])?;
    uni.delta(right, &Hex::from(x))?;
    uni.put(c, "x", right)?;
    Ok(c)
}

#[test]
fn adds_two_integers() -> Result<()> {
    let mut uni = Universe::empty();
    let c = make_call(&mut uni, 40, 2)?;
    let result = int_plus(&mut uni, c)?;
    assert_eq!(42, uni.dataize(format!("ν{result}").as_str())?.to_i64()?);
    Ok(())
}

#[test]
fn multiplies_two_integers() -> Result<()> {
    let mut uni = Universe::empty();
    let c = make_call(&mut uni, 6, 7)?;
    let result = int_times(&mut uni, c)?;
    assert_eq!(42, uni.dataize(format!("ν{result}").as_str())?.to_i64()?);
    Ok(())
}

#[test]
fn fails_when_plus_overflows() -> Result<()> {
    let mut uni = Universe::empty();
    let c = make_call(&mut uni, i64::MAX, 1)?;
    assert!(
        int_plus(&mut uni, c).is_err(),
        "Overflowing addition must return an error, not panic or wrap around"
    );
    Ok(())
}

#[test]
fn fails_when_times_overflows() -> Result<()> {
    let mut uni = Universe::empty();
    let c = make_call(&mut uni, i64::MAX, 2)?;
    assert!(
        int_times(&mut uni, c).is_err(),
        "Overflowing multiplication must return an error, not panic or wrap around"
    );
    Ok(())
}

#[test]
fn divides_two_integers() -> Result<()> {
    let mut uni = Universe::empty();
    let c = make_call(&mut uni, 12, 4)?;
    let result = int_div(&mut uni, c)?;
    assert_eq!(3, uni.dataize(format!("ν{result}").as_str())?.to_i64()?);
    Ok(())
}

#[test]
fn fails_to_divide_by_zero() -> Result<()> {
    let mut uni = Universe::empty();
    let c = make_call(&mut uni, 42, 0)?;
    assert!(
        int_div(&mut uni, c).is_err(),
        "Division by zero must return an error, not panic"
    );
    Ok(())
}

#[test]
fn fails_when_div_overflows() -> Result<()> {
    let mut uni = Universe::empty();
    let c = make_call(&mut uni, i64::MIN, -1)?;
    assert!(
        int_div(&mut uni, c).is_err(),
        "Overflowing division must return an error, not panic"
    );
    Ok(())
}

#[test]
fn dataizes_atom_from_phino_script() -> Result<()> {
    let g = crate::deploy(
        "
        F c0 int foo
        F c1 plus φ
        F c2 ρ x
        λ c2 L_int_plus
        P c1 plus c2
        P c0 int c1
        F c3
        Δ c3 00-00-00-00-00-00-00-28
        P c1 φ c3
        D c4 c0 int
        D c5 c4 plus
        F c6
        Δ c6 00-00-00-00-00-00-00-02
        A c7 c5 x c6
        P c0 foo c7
        ",
    )?;
    let mut uni = Universe::from_graph(g)?;
    crate::org::eolang::register(&mut uni);
    assert_eq!(42, uni.dataize("Φ.foo")?.to_i64()?);
    Ok(())
}
