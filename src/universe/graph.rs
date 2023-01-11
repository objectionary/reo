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

use crate::universe::Universe;
use anyhow::Result;
use itertools::Itertools;
use xml_builder::{XMLBuilder, XMLElement, XMLVersion};

impl Universe {
    /// Make XML graph.
    pub fn to_graph(&self) -> Result<String> {
        let mut xml = XMLBuilder::new()
            .version(XMLVersion::XML1_1)
            .encoding("UTF-8".into())
            .build();
        let mut root = XMLElement::new("graph");
        for (v, vtx) in self
            .vertices
            .iter()
            .sorted_by_key(|(v, _)| <&u32>::clone(v))
        {
            let mut v_node = XMLElement::new("v");
            v_node.add_attribute("id", v.to_string().as_str());
            for (e, edge) in self
                .edges
                .iter()
                .filter(|(_, edge)| edge.from == *v)
                .sorted_by_key(|(e, _)| <&u32>::clone(e))
            {
                let mut e_node = XMLElement::new("e");
                e_node.add_attribute("id", e.to_string().as_str());
                e_node.add_attribute("title", edge.a.as_str());
                e_node.add_attribute("to", edge.to.to_string().as_str());
                v_node.add_child(e_node).unwrap();
            }
            if let Some(d) = &(*vtx).data {
                let mut data_node = XMLElement::new("data");
                data_node
                    .add_text(format!("bytes/{}", d.as_hex().replace(' ', "-")))
                    .unwrap();
                v_node.add_child(data_node).unwrap();
            }
            if !vtx.lambda_name.is_empty() {
                let mut lambda_node = XMLElement::new("lambda");
                lambda_node
                    .add_text((*vtx).lambda_name.to_string())
                    .unwrap();
                v_node.add_child(lambda_node).unwrap();
            }
            root.add_child(v_node).unwrap();
        }
        xml.set_root_element(root);
        let mut writer: Vec<u8> = Vec::new();
        xml.generate(&mut writer).unwrap();
        Ok(std::str::from_utf8(&writer)?.to_string())
    }
}

#[cfg(test)]
use crate::data::Data;

#[cfg(test)]
use sxd_xpath::evaluate_xpath;

#[test]
fn prints_simple_graph() -> Result<()> {
    let mut uni = Universe::empty();
    uni.add(0)?;
    uni.data(0, Data::from_int(42))?;
    uni.add(1)?;
    uni.bind(1, 0, 1, "foo")?;
    uni.atom(1, "S/Q")?;
    let xml = uni.to_graph()?;
    println!("{}", xml);
    let parser = sxd_document::parser::parse(xml.as_str())?;
    let doc = parser.as_document();
    assert_eq!(
        "ρ",
        evaluate_xpath(&doc, "/graph/v[@id=1]/e[@id=2]/@title")?.string()
    );
    assert_eq!(
        "S/Q",
        evaluate_xpath(&doc, "/graph/v[@id=1]/lambda")?.string()
    );
    assert_eq!(
        "bytes/00-00-00-00-00-00-00-2A",
        evaluate_xpath(&doc, "/graph/v[@id=0]/data")?.string()
    );
    Ok(())
}
