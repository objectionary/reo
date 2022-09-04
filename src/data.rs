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

pub struct Data {
    bytes: Vec<u8>,
}

impl Clone for Data {
    fn clone(&self) -> Self {
        Data::from_bytes(self.bytes.clone())
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

    pub fn as_int(&self) -> Result<i64> {
        let a: &[u8; 8] = &self
            .bytes
            .as_slice()
            .try_into()
            .context(format!("There is no data, can't make INT"))?;
        Ok(i64::from_be_bytes(*a))
    }

    pub fn as_float(&self) -> Result<f64> {
        let a: &[u8; 8] = &self
            .bytes
            .as_slice()
            .try_into()
            .context(format!("There is no data, can't make FLOAT"))?;
        Ok(f64::from_be_bytes(*a))
    }

    pub fn as_string(&self) -> Result<String> {
        Ok(String::from_utf8(self.bytes.clone())?)
    }

    pub fn as_hex(&self) -> String {
        if self.bytes.is_empty() {
            "--".to_string()
        } else {
            self.bytes
                .iter()
                .map(|b| format!("{:02x}", b).to_string())
                .collect::<Vec<String>>()
                .join("-")
        }
    }
}

#[test]
fn simple_int() {
    let i = 42;
    let d = Data::from_int(42);
    assert_eq!(i, d.as_int().unwrap());
}

#[test]
fn prints_bytes() {
    let txt = "привет";
    let d = Data::from_str(txt);
    assert_eq!("d0-bf-d1-80-d0-b8-d0-b2-d0-b5-d1-82", d.as_hex());
    assert_eq!(txt, Data::from_hex(d.as_hex()).as_string().unwrap());
}

#[test]
fn prints_empty_bytes() {
    let txt = "";
    let d = Data::from_str(txt);
    assert_eq!("--", d.as_hex());
}
