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

use anyhow::Result;
use reo::da;
use reo::gmi::Gmi;
use reo::universe::Universe;
use tempfile::TempDir;

#[test]
fn link() -> Result<()> {
    let tmp = TempDir::new()?;
    let relf = tmp.path().join("temp.relf");
    let uni1 = &mut Universe::empty();
    Gmi::from_string(
        "
        ADD('ν0');
        ADD('$ν1');
        BIND('$ε1', 'ν0', '$ν1', 'foo');
        DATA('$ν1', 'd0 bf d1 80 d0 b8 d0 b2 d0 b5 d1 82');
        "
        .to_string(),
    )?
    .deploy_to(uni1)?;
    uni1.save(relf.as_path())?;
    let before1 = da!(uni1, "Φ.foo").as_string()?;
    let target = tmp.path().join("target.relf");
    let uni2 = &mut Universe::empty();
    Gmi::from_string(
        "
        ADD('ν0');
        ADD('$ν1');
        BIND('ε2', 'ν0', '$ν1', 'bar');
        DATA('$ν1', 'd0 bc d0 b8 d1 80');
        "
        .to_string(),
    )?
    .deploy_to(uni2)?;
    uni2.save(target.as_path())?;
    let before2 = da!(uni2, "Φ.bar").as_string()?;
    assert_cmd::Command::cargo_bin("reo")
        .unwrap()
        .arg("link")
        .arg(target.as_os_str())
        .arg(relf.as_os_str())
        .assert()
        .success();
    let mut uni = Universe::load(target.as_path())?;
    let after1 = da!(uni, "Φ.foo").as_string()?;
    let after2 = da!(uni, "Φ.bar").as_string()?;
    assert_eq!(before1, after1);
    assert_eq!(before2, after2);
    Ok(())
}
