// SPDX-FileCopyrightText: Copyright (c) 2022-2026 Yegor Bugayenko
// SPDX-License-Identifier: MIT

//! Tests of the atoms, on a tiny runtime written by hand, in the
//! same shape as `phino sodg` prints the objects of eo-runtime.

use crate::graph::deploy;
use crate::org::eolang::data::{bytes_of, number_of, string_of};
use crate::org::eolang::register;
use crate::Universe;
use anyhow::Result;
use sodg::Hex;

/// A method with the atom, in the formation `o`.
fn atom(o: &str, name: &str, voids: &str, lambda: &str) -> String {
    format!("F {o}{name} ρ {voids}\nλ {o}{name} {lambda}\nP {o} {name} {o}{name}\n")
}

/// The tiny runtime with `number`, `bytes`, `string`, `true`, and
/// `false`, and the root attribute `foo` to put the object under test to.
fn runtime() -> String {
    let mut txt = String::from("F c0 number bytes string true false dataized foo\n");
    txt.push_str("F n φ plus times div gt\nP c0 number n\n");
    for (a, l) in [
        ("plus", "plus"),
        ("times", "times"),
        ("div", "div"),
        ("gt", "gt"),
    ] {
        txt.push_str(&atom("n", a, "v🌵b", &format!("L_number_{l}")));
    }
    txt.push_str("F b φ and or not concat eq left right size slice as-bytes\nP c0 bytes b\n");
    for a in ["and", "or", "concat", "eq", "left", "right"] {
        txt.push_str(&atom("b", a, "v🌵b", &format!("L_bytes_{a}")));
    }
    for a in ["not", "size"] {
        txt.push_str(&atom("b", a, "", &format!("L_bytes_{a}")));
    }
    txt.push_str(&atom(
        "b",
        "slice",
        "v🌵start v🌵len v🌵cant-slice",
        "L_bytes_slice",
    ));
    txt.push_str("F bas ρ φ\nD bas_r bas ρ\nP bas φ bas_r\nP b as-bytes bas\n");
    txt.push_str("F s φ\nP c0 string s\n");
    txt.push_str("F t\nΔ t 01\nP c0 true t\nF f\nΔ f 00\nP c0 false f\n");
    txt.push_str("F d v🌵target\nλ d L_dataized\nP c0 dataized d\n");
    txt
}

/// Make a Universe with the tiny runtime and all atoms registered.
fn universe() -> Result<Universe> {
    let mut uni = Universe::from_graph(deploy(&runtime())?)?;
    register(&mut uni);
    Ok(uni)
}

/// Put `ρ.method(args...)` to `Φ.foo` and dataize it.
fn call(uni: &mut Universe, rho: usize, method: &str, args: &[(&str, usize)]) -> Result<Hex> {
    let mut c = uni.dispatch(rho, method)?;
    for (a, v) in args {
        c = uni.application(c, a, *v)?;
    }
    uni.put(0, "foo", c)?;
    uni.dataize("Φ.foo")
}

fn num(h: &Hex) -> f64 {
    f64::from_be_bytes(h.bytes().try_into().unwrap())
}

#[test]
fn adds_multiplies_and_divides_numbers() -> Result<()> {
    let mut uni = universe()?;
    for (method, expected) in [("plus", 8.5), ("times", 15.0), ("div", 2.4)] {
        let x = number_of(&mut uni, 6.0)?;
        let y = number_of(&mut uni, 2.5)?;
        let r = call(&mut uni, x, method, &[("v🌵b", y)])?;
        assert!(
            (num(&r) - expected).abs() < f64::EPSILON,
            "{method}: {}",
            num(&r)
        );
    }
    Ok(())
}

#[test]
fn divides_by_zero_into_infinity() -> Result<()> {
    let mut uni = universe()?;
    let x = number_of(&mut uni, 1.0)?;
    let y = number_of(&mut uni, 0.0)?;
    assert!(num(&call(&mut uni, x, "div", &[("v🌵b", y)])?).is_infinite());
    Ok(())
}

#[test]
fn compares_numbers() -> Result<()> {
    let mut uni = universe()?;
    for (a, b, expected) in [(3.0, 2.0, 1_u8), (2.0, 3.0, 0), (2.0, 2.0, 0)] {
        let x = number_of(&mut uni, a)?;
        let y = number_of(&mut uni, b)?;
        assert_eq!(
            &[expected],
            call(&mut uni, x, "gt", &[("v🌵b", y)])?.bytes()
        );
    }
    Ok(())
}

#[test]
fn applies_bitwise_operations() -> Result<()> {
    let mut uni = universe()?;
    for (method, expected) in [("and", 0x0C_u8), ("or", 0x3F)] {
        let x = bytes_of(&mut uni, &[0x0F])?;
        let y = bytes_of(&mut uni, &[0x3C])?;
        assert_eq!(
            &[expected],
            call(&mut uni, x, method, &[("v🌵b", y)])?.bytes()
        );
    }
    let x = bytes_of(&mut uni, &[0x0F, 0x00])?;
    assert_eq!(&[0xF0, 0xFF], call(&mut uni, x, "not", &[])?.bytes());
    Ok(())
}

#[test]
fn rejects_bitwise_operation_on_different_sizes() -> Result<()> {
    let mut uni = universe()?;
    let x = bytes_of(&mut uni, &[0x0F])?;
    let y = bytes_of(&mut uni, &[0x3C, 0x00])?;
    let err = call(&mut uni, x, "and", &[("v🌵b", y)]).unwrap_err();
    assert!(format!("{err:#}").contains("same length"), "{err:#}");
    Ok(())
}

#[test]
fn concatenates_and_measures_bytes() -> Result<()> {
    let mut uni = universe()?;
    let x = bytes_of(&mut uni, &[0x01, 0x02])?;
    let y = bytes_of(&mut uni, &[0x03])?;
    assert_eq!(
        &[0x01, 0x02, 0x03],
        call(&mut uni, x, "concat", &[("v🌵b", y)])?.bytes()
    );
    let x = bytes_of(&mut uni, &[0x01, 0x02, 0x03])?;
    assert!((num(&call(&mut uni, x, "size", &[])?) - 3.0).abs() < f64::EPSILON);
    Ok(())
}

#[test]
fn compares_bytes() -> Result<()> {
    let mut uni = universe()?;
    for (other, expected) in [(0x2A_u8, 1_u8), (0x2B, 0)] {
        let x = bytes_of(&mut uni, &[0x2A])?;
        let y = bytes_of(&mut uni, &[other])?;
        assert_eq!(
            &[expected],
            call(&mut uni, x, "eq", &[("v🌵b", y)])?.bytes()
        );
    }
    Ok(())
}

#[test]
fn shifts_bytes() -> Result<()> {
    let mut uni = universe()?;
    for (method, bits, expected) in [("left", 4.0, [0x23, 0x40]), ("right", 4.0, [0x01, 0x23])] {
        let x = bytes_of(&mut uni, &[0x12, 0x34])?;
        let y = number_of(&mut uni, bits)?;
        assert_eq!(
            &expected,
            call(&mut uni, x, method, &[("v🌵b", y)])?.bytes()
        );
    }
    Ok(())
}

#[test]
fn slices_bytes() -> Result<()> {
    let mut uni = universe()?;
    let x = bytes_of(&mut uni, &[0x01, 0x02, 0x03, 0x04])?;
    let start = number_of(&mut uni, 1.0)?;
    let len = number_of(&mut uni, 2.0)?;
    let fallback = uni.formation(&[])?;
    let r = call(
        &mut uni,
        x,
        "slice",
        &[
            ("v🌵start", start),
            ("v🌵len", len),
            ("v🌵cant-slice", fallback),
        ],
    )?;
    assert_eq!(&[0x02, 0x03], r.bytes());
    Ok(())
}

#[test]
fn gives_fallback_on_out_of_bounds_slice() -> Result<()> {
    let mut uni = universe()?;
    let x = bytes_of(&mut uni, &[0x01])?;
    let start = number_of(&mut uni, 1.0)?;
    let len = number_of(&mut uni, 2.0)?;
    let fallback = uni.formation(&["msg", "φ"])?;
    let phi = uni.dispatch(fallback, "msg")?;
    uni.put(fallback, "φ", phi)?;
    let r = call(
        &mut uni,
        x,
        "slice",
        &[
            ("v🌵start", start),
            ("v🌵len", len),
            ("v🌵cant-slice", fallback),
        ],
    )?;
    assert_eq!(
        "cannot slice '2' bytes from offset '1' of bytes of size 1",
        r.to_utf8()?
    );
    Ok(())
}

#[test]
fn dataizes_target() -> Result<()> {
    let mut uni = universe()?;
    let txt = string_of(&mut uni, "hello")?;
    let d = uni.dispatch(0, "dataized")?;
    let c = uni.application(d, "v🌵target", txt)?;
    uni.put(0, "foo", c)?;
    assert_eq!("hello", uni.dataize("Φ.foo")?.to_utf8()?);
    Ok(())
}
