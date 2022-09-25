// Copyright (c) 2022 Yegor Bugayenko
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included
// in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NON-INFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};

#[derive(Serialize, Deserialize)]
pub struct Data {
    bytes: Vec<u8>,
}

impl Clone for Data {
    fn clone(&self) -> Self {
        Data::from_bytes(self.bytes.clone())
    }
}

impl Debug for Data {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_hex().as_str())
    }
}

impl PartialEq for Data {
    fn eq(&self, other: &Self) -> bool {
        self.as_hex() == other.as_hex()
    }
}

impl Data {
    pub fn empty() -> Self {
        Self::from_bytes(Vec::new())
    }

    /// From BYTES.
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Data { bytes }
    }

    /// From BYTES as HEX.
    pub fn from_hex(hex: String) -> Self {
        let s = hex.replace("-", "");
        Self::from_bytes(hex::decode(s).unwrap())
    }

    /// From INT.
    pub fn from_int(d: i64) -> Self {
        Self::from_bytes(d.to_be_bytes().to_vec())
    }

    /// From BOOL.
    pub fn from_bool(d: bool) -> Self {
        Self::from_bytes(if d { [1] } else { [0] }.to_vec())
    }

    /// From FLOAT.
    pub fn from_float(d: f64) -> Self {
        Self::from_bytes(d.to_be_bytes().to_vec())
    }

    /// From STRING.
    pub fn from_string(d: String) -> Self {
        Self::from_bytes(d.as_bytes().to_vec())
    }

    /// From STR.
    pub fn from_str(d: &str) -> Self {
        Self::from_bytes(d.to_string().as_bytes().to_vec())
    }

    /// It's empty and no data?
    pub fn is_empty(&self) -> bool {
        self.bytes.len() == 0
    }

    /// Turn it into `bool`.
    ///
    /// ```
    /// use reo::data::Data;
    /// let d = Data::from_bytes([0x01].to_vec());
    /// assert_eq!(true, d.as_bool().unwrap());
    /// ```
    pub fn as_bool(&self) -> Result<bool> {
        Ok(self.bytes[0] == 0x01)
    }

    /// Turn it into `i64`.
    ///
    /// ```
    /// use reo::data::Data;
    /// let d = Data::from_bytes([0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2A].to_vec());
    /// assert_eq!(42, d.as_int().unwrap());
    /// ```
    pub fn as_int(&self) -> Result<i64> {
        let a: &[u8; 8] = &self
            .bytes
            .as_slice()
            .try_into()
            .context(format!("There is no data, can't make INT"))?;
        Ok(i64::from_be_bytes(*a))
    }

    /// Turn it into `f64`.
    ///
    /// ```
    /// use reo::data::Data;
    /// let d = Data::from_bytes([0x40, 0x09, 0x21, 0xfb, 0x54, 0x44, 0x2d, 0x18].to_vec());
    /// assert_eq!(std::f64::consts::PI, d.as_float().unwrap());
    /// ```
    pub fn as_float(&self) -> Result<f64> {
        let a: &[u8; 8] = &self
            .bytes
            .as_slice()
            .try_into()
            .context(format!("There is no data, can't make FLOAT"))?;
        Ok(f64::from_be_bytes(*a))
    }

    /// Turn it into `string`.
    ///
    /// ```
    /// use reo::data::Data;
    /// let d = Data::from_bytes([0x41, 0x42].to_vec());
    /// assert_eq!("AB", d.as_string().unwrap());
    /// ```
    pub fn as_string(&self) -> Result<String> {
        Ok(String::from_utf8(self.bytes.clone())?)
    }

    /// Turn it into a hexadecimal string.
    ///
    /// ```
    /// use reo::data::Data;
    /// let d = Data::from_bytes([0xCA, 0xFE].to_vec());
    /// assert_eq!("CA-FE", d.as_hex());
    /// ```
    pub fn as_hex(&self) -> String {
        if self.bytes.is_empty() {
            "--".to_string()
        } else {
            self.bytes
                .iter()
                .map(|b| format!("{:02X}", b).to_string())
                .collect::<Vec<String>>()
                .join("-")
        }
    }

    /// Turn it into a vector of bytes.
    pub fn as_bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }
}

#[test]
fn simple_int() {
    let i = 42;
    let d = Data::from_int(i);
    assert_eq!(i, d.as_int().unwrap());
    assert_eq!("00-00-00-00-00-00-00-2A", d.as_hex());
}

#[test]
fn simple_bool() {
    let b = true;
    let d = Data::from_bool(b);
    assert_eq!(b, d.as_bool().unwrap());
    assert_eq!("01", d.as_hex());
}

#[test]
fn simple_float() {
    let f = std::f64::consts::PI;
    let d = Data::from_float(f);
    assert_eq!(f, d.as_float().unwrap());
    assert_eq!("40-09-21-FB-54-44-2D-18", d.as_hex());
}

#[test]
fn compares_with_data() {
    let i = 42;
    let left = Data::from_int(i);
    let right = Data::from_int(i);
    assert_eq!(left, right);
}

#[test]
fn prints_bytes() {
    let txt = "привет";
    let d = Data::from_str(txt);
    assert_eq!("D0-BF-D1-80-D0-B8-D0-B2-D0-B5-D1-82", d.as_hex());
    assert_eq!(txt, Data::from_hex(d.as_hex()).as_string().unwrap());
}

#[test]
fn prints_empty_bytes() {
    let txt = "";
    let d = Data::from_str(txt);
    assert_eq!("--", d.as_hex());
}

#[test]
fn broken_int_from_small_data() {
    let d = Data::from_bytes([0x01, 0x02].to_vec());
    let ret = d.as_int();
    assert!(ret.is_err());
}

#[test]
fn broken_float_from_small_data() {
    let d = Data::from_bytes([0x00].to_vec());
    let ret = d.as_float();
    assert!(ret.is_err());
}
