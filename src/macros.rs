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

#[macro_export]
macro_rules! da {
    ($uni:expr, $loc:expr) => {
        $uni.dataize(format!("{}.Δ", $loc).as_str())
            .expect(format!("Can't dataize {}", $loc).as_str())
    };
}

/// Add a new vertex to the universe and return its ID:
///
/// ```
/// use reo::universe::Universe;
/// use reo::add;
/// let mut uni = Universe::empty();
/// let v1 = add!(uni);
/// ```
#[macro_export]
macro_rules! add {
    ($uni:expr) => {{
        let v = $uni.next_v();
        $uni.add(v).unwrap();
        v
    }};
}

/// Adds a new edge between two vertices:
///
/// ```
/// use reo::universe::Universe;
/// use reo::{add, bind};
/// let mut uni = Universe::empty();
/// let v1 = add!(uni);
/// let v2 = add!(uni);
/// let e = bind!(uni, v1, v2, "foo");
/// ```
#[macro_export]
macro_rules! bind {
    ($uni:expr, $v1:expr, $v2:expr, $a:expr) => {{
        let e = $uni.next_e();
        $uni.bind(e, $v1, $v2, $a).unwrap();
        e
    }};
}

/// Makes a copy of an existing vertex, by looking at the edge
/// that leads to it:
///
/// ```
/// use reo::universe::Universe;
/// use reo::{add, bind, copy};
/// let mut uni = Universe::empty();
/// let v1 = add!(uni);
/// let v2 = add!(uni);
/// let e = bind!(uni, v1, v2, "foo");
/// copy!(uni, e);
/// ```
#[macro_export]
macro_rules! copy {
    ($uni:expr, $e1:expr) => {{
        let v3 = $uni.next_v();
        let e2 = $uni.next_e();
        $uni.copy($e1, v3, e2).unwrap();
        v3
    }};
}
