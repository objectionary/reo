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

pub struct Data {
    bytes: Vec<u8>
}

impl Data {
    pub fn empty() -> Self {
        Self::from_bytes(Vec::new())
    }

    // From INT.
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Data { bytes }
    }

    // From INT.
    pub fn from_int(d: u64) -> Self {
        Self::from_bytes(d.to_be_bytes().to_vec())
    }

    // From FLOAT.
    pub fn from_float(d: f64) -> Self  {
        Self::from_bytes(d.to_be_bytes().to_vec())
    }

    // From STRING.
    pub fn from_string(d: String) -> Self {
        Self::from_bytes(d.as_bytes().to_vec())
    }

    pub fn as_int(&self) -> u64 {
        let a : &[u8; 8] = &self.bytes.clone().try_into().unwrap();
        u64::from_be_bytes(*a)
    }

    pub fn as_float(&self) -> f64 {
        let a : &[u8; 8] = &self.bytes.clone().try_into().unwrap();
        f64::from_be_bytes(*a)
    }

    pub fn as_string(&self) -> String {
        String::from_utf8(self.bytes.clone()).unwrap()
    }

}

#[test]
fn simple_int() {
    let i = 42;
    let d = Data::from_int(42);
    assert_eq!(i, d.as_int());
}
