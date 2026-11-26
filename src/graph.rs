// SPDX-FileCopyrightText: Copyright (c) 2022-2026 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use anyhow::{anyhow, bail, Context, Result};
use log::debug;
use sodg::{Kind, Label, Script, Sodg};
use std::cmp;
use std::collections::{HashMap, HashSet};

/// The maximum number of edges a single cell of the [`Graph`] may have.
///
/// A formation needs an edge per attribute, while a closed one may also
/// get an edge per attribute memoized after it was found through `φ`
/// or an atom.
pub const EDGES: usize = 256;

/// The graph of objects, which [`crate::Universe`] evaluates.
pub type Graph = Sodg<EDGES>;

/// The smallest capacity of a [`Graph`] made here.
const MIN_CAPACITY: usize = 16;

/// The edge from a dispatch or an application to its base or proto.
pub(crate) const PI: Label = Label::Greek('π');

/// The attribute of a decorated object.
pub(crate) const PHI: Label = Label::Greek('φ');

/// The attribute of a parent object.
pub(crate) const RHO: Label = Label::Greek('ρ');

/// Deploy the text printed by `phino sodg` to a new [`Graph`].
///
/// The graph is checked by [`validate`], so a script with positional
/// attributes, like `α0`, is rejected:
///
/// ```
/// let g = reo::deploy("F c0 foo\nF c1\nΔ c1 2A\nP c0 foo c1").unwrap();
/// assert_eq!(2, g.len());
/// assert!(reo::deploy("F c0 α0").is_err());
/// ```
///
/// The λ-asset `Package`, which `phino sodg` gives to a package, is
/// not deployed, since for REO a package is just a formation. Otherwise,
/// when the package `Φ.number` from `number/abs.sodg` is merged with
/// the object `Φ.number` from `number.sodg`, the object gets the atom
/// `Package` and can't be dataized:
///
/// ```
/// let g = reo::deploy("F c0 number\nF c1 abs\nλ c1 Package\nP c0 number c1").unwrap();
/// assert_eq!(None, g.atom(1));
/// ```
///
/// # Errors
///
/// If the script can't be parsed, uses positional attributes,
/// or gives a cell more than [`EDGES`] edges.
pub fn deploy(txt: &str) -> Result<Graph> {
    check_edges(txt)?;
    let mut g = Graph::empty(capacity_for(txt));
    let total = Script::from_str(&without_packages(txt)).deploy_to(&mut g)?;
    debug!("Deployed {total} commands into {} cells", g.len());
    validate(&g)?;
    Ok(g)
}

/// Remove the commands `λ c Package` from the script, keeping the
/// numbers of all other lines, so that errors still point to them.
fn without_packages(txt: &str) -> String {
    txt.lines()
        .map(|line| {
            let cmd = line.split_once('#').map_or(line, |(before, _)| before);
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            if matches!(parts.as_slice(), ["λ", _, "Package"]) {
                ""
            } else {
                line
            }
        })
        .collect::<Vec<&str>>()
        .join("\n")
}

/// Make sure the graph doesn't use positional attributes, like `α0`,
/// which REO doesn't support.
///
/// # Errors
///
/// If a positional attribute is found, the error names the cell.
pub fn validate(g: &Graph) -> Result<()> {
    let mut keys = g.keys();
    keys.sort_unstable();
    for c in keys {
        let mut labels: Vec<Label> = g.attrs(c).to_vec();
        labels.extend(g.kids(c).map(|(a, _)| *a));
        match g.kind(c) {
            Kind::Dispatch(a) | Kind::Application(a) => labels.push(a),
            Kind::Formation | Kind::Termination | Kind::Void => {}
        }
        if let Some(a) = labels.iter().find(|a| matches!(a, Label::Alpha(_))) {
            bail!("The positional attribute {a} of c{c} is not supported, use a named one");
        }
    }
    Ok(())
}

/// Make a copy of the graph with the capacity `cap`, which must
/// be larger than any ID of a cell in it.
///
/// Every made cell keeps its ID, its kind, its attributes, its edges,
/// and its Δ- and λ-assets.
pub(crate) fn rebuilt(g: &Graph, cap: usize) -> Graph {
    let mut ng = Graph::empty(cap);
    let keys = g.keys();
    for c in &keys {
        let c = *c;
        match g.kind(c) {
            Kind::Formation => ng.formation(c, g.attrs(c)),
            Kind::Dispatch(a) => ng.dispatch(c, g.kid(c, PI).unwrap_or(c), a),
            Kind::Application(a) => {
                ng.application(c, g.kid(c, PI).unwrap_or(c), a, g.kid(c, a).unwrap_or(c))
            }
            Kind::Termination => ng.termination(c),
            Kind::Void => {}
        }
        if let Some(d) = g.data(c) {
            ng.delta(c, &d);
        }
        if let Some(name) = g.atom(c) {
            ng.lambda(c, name);
        }
        for (a, k) in g.kids(c) {
            ng.put(c, *a, *k);
        }
    }
    ng
}

/// The smallest ID that is larger than the ID of every cell in the graph.
pub(crate) fn next_after(g: &Graph) -> usize {
    g.keys().into_iter().max().map_or(0, |m| m + 1)
}

/// Merge the graph `right` into the graph `left`, matching their roots.
///
/// The attributes of the cells that were in `left` before the merge
/// are joined with the attributes of the cells merged into them.
///
/// # Errors
///
/// If the graphs can't be merged, for example, when a cell
/// would get more than [`EDGES`] edges.
pub fn merge(left: &Graph, right: &Graph) -> Result<Graph> {
    if left.kind(0) == Kind::Void {
        return Err(anyhow!("There is no root c0 in the graph to merge into"));
    }
    check_merge(left, right)?;
    let mut g = rebuilt(left, next_after(left) + right.len() + MIN_CAPACITY);
    let before: Vec<(usize, Vec<Label>)> = left
        .keys()
        .into_iter()
        .filter(|c| left.kind(*c) == Kind::Formation)
        .map(|c| (c, left.attrs(c).to_vec()))
        .collect();
    g.merge(right, 0, 0)
        .context("Failed to merge the graphs, maybe they are not trees")?;
    for (c, attrs) in before {
        if g.kind(c) != Kind::Formation {
            continue;
        }
        let mut seen: HashSet<Label> = HashSet::new();
        let union: Vec<Label> = attrs
            .iter()
            .chain(g.attrs(c).iter())
            .filter(|a| seen.insert(**a))
            .copied()
            .collect();
        g.formation(c, &union);
    }
    validate(&g)?;
    Ok(g)
}

/// Make sure no cell gets more than [`EDGES`] edges from the script,
/// since [`Sodg`] panics when a cell has no room for one more edge.
fn check_edges(txt: &str) -> Result<()> {
    let mut edges: HashMap<&str, HashSet<&str>> = HashMap::new();
    for line in txt.lines() {
        let cmd = line.split_once('#').map_or(line, |(before, _)| before);
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        match parts.as_slice() {
            ["P", c, a, _] => {
                edges.entry(c).or_default().insert(a);
            }
            ["D", c, _, _] => {
                edges.entry(c).or_default().insert("π");
            }
            ["A", c, _, a, _] => {
                let e = edges.entry(c).or_default();
                e.insert("π");
                e.insert(a);
            }
            _ => {}
        }
    }
    if let Some((c, e)) = edges.iter().find(|(_, e)| e.len() > EDGES) {
        bail!(
            "The cell {c} has {} edges, while no more than {EDGES} are allowed",
            e.len()
        );
    }
    Ok(())
}

/// Make sure no cell of `left` gets more than [`EDGES`] edges, when
/// `right` is merged into it, since [`Sodg`] panics when a cell
/// has no room for one more edge.
///
/// The cells of `right` are matched to the cells of `left` the same way
/// [`Sodg::merge`] does it: the roots are matched and then the kids
/// of matched cells are matched by their attributes.
fn check_merge(left: &Graph, right: &Graph) -> Result<()> {
    let mut seen = HashSet::new();
    let mut todo = vec![(0, 0)];
    while let Some((r, l)) = todo.pop() {
        if !seen.insert(r) {
            continue;
        }
        let mut labels: HashSet<Label> = left.kids(l).map(|(a, _)| *a).collect();
        for (a, k) in right.kids(r) {
            labels.insert(*a);
            if let Some(lk) = left.kid(l, *a) {
                todo.push((*k, lk));
            }
        }
        if labels.len() > EDGES {
            bail!(
                "The cell c{l} would have {} edges after the merge, while no more than {EDGES} are allowed",
                labels.len()
            );
        }
    }
    Ok(())
}

/// Calculate the capacity enough to deploy the script: every cell
/// is either a name or a number, and every name takes no more than
/// one new ID.
fn capacity_for(txt: &str) -> usize {
    let words = txt.split_whitespace();
    let total = words.clone().count();
    let biggest = words
        .filter_map(|w| w.parse::<usize>().ok())
        .max()
        .unwrap_or(0);
    cmp::max(MIN_CAPACITY, total + biggest + 1)
}

#[cfg(test)]
use std::str::FromStr;

#[test]
fn deploys_simple_script() -> Result<()> {
    let g = deploy("F c0 foo\nF c1\nΔ c1 00-2A\nP c0 foo c1")?;
    assert_eq!(2, g.len());
    assert_eq!(Some(1), g.kid(0, Label::from_str("foo")?));
    Ok(())
}

#[test]
fn rejects_positional_attribute() {
    let err = deploy("F c0 foo\nD c1 c0 x\nF c2\nA c3 c1 α0 c2\nP c0 foo c3").unwrap_err();
    assert!(
        err.to_string().contains("α0 of c3"),
        "Unexpected error: {err}"
    );
}

#[test]
fn rejects_positional_formation_attribute() {
    assert!(deploy("F c0 α1").is_err());
}

#[test]
fn deploys_script_with_big_numbers() -> Result<()> {
    let g = deploy("F 0 foo\nF 100\nP 0 foo 100")?;
    assert_eq!(2, g.len());
    Ok(())
}

#[test]
fn rebuilds_with_larger_capacity() -> Result<()> {
    let g = deploy(
        "F c0 x y\nD c1 c0 x\nF c2\nA c3 c1 z c2\nΔ c2 00-00-00-00-00-00-00-2A\nλ c0 L_foo\nP c0 y c3",
    )?;
    let ng = rebuilt(&g, 1024);
    assert_eq!(g.len(), ng.len());
    assert_eq!(Kind::Application(Label::from_str("z")?), ng.kind(3));
    assert_eq!(Some(1), ng.kid(3, PI));
    assert_eq!(Some(2), ng.kid(3, Label::from_str("z")?));
    assert_eq!(Some("L_foo"), ng.atom(0));
    assert_eq!(42, ng.data(2).unwrap().to_i64()?);
    assert_eq!(Some(3), ng.kid(0, Label::from_str("y")?));
    Ok(())
}

#[test]
fn merges_two_graphs() -> Result<()> {
    let left = deploy("F c0 foo\nF c1\nΔ c1 2A\nP c0 foo c1")?;
    let right = deploy("F c0 bar\nF c1\nΔ c1 2B\nP c0 bar c1")?;
    let g = merge(&left, &right)?;
    assert_eq!(3, g.len());
    assert_eq!(
        vec!["foo", "bar"],
        g.attrs(0)
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[test]
fn merges_graphs_with_dispatches_of_root() -> Result<()> {
    let left = deploy("F c0 bar\nF c1\nΔ c1 00-00-00-00-00-00-00-2A\nP c0 bar c1")?;
    let right = deploy("F c0 foo\nD c1 c0 bar\nP c0 foo c1")?;
    let mut empty = Graph::empty(MIN_CAPACITY);
    empty.formation(0, &[]);
    let g = merge(&merge(&empty, &left)?, &right)?;
    let mut uni = crate::Universe::from_graph(g)?;
    assert_eq!(42, uni.dataize("Φ.foo")?.to_i64()?);
    Ok(())
}

#[test]
fn merges_package_into_object_of_the_same_name() -> Result<()> {
    let object =
        deploy("F c0 num\nF c1 φ\nF c2\nΔ c2 00-00-00-00-00-00-00-2A\nP c1 φ c2\nP c0 num c1")?;
    let package =
        deploy("F c0 num\nF c1 abs # the package\nλ c1 Package\nF c2\nP c1 abs c2\nP c0 num c1")?;
    let mut empty = Graph::empty(MIN_CAPACITY);
    empty.formation(0, &[]);
    for g in [
        merge(&merge(&empty, &object)?, &package)?,
        merge(&merge(&empty, &package)?, &object)?,
    ] {
        let num = g.kid(0, Label::from_str("num")?).unwrap();
        assert_eq!(None, g.atom(num));
        let mut uni = crate::Universe::from_graph(g)?;
        assert_eq!(42, uni.dataize("Φ.num")?.to_i64()?);
    }
    Ok(())
}

#[test]
fn keeps_line_numbers_without_packages() {
    let err = deploy("F c0 x\nλ c0 Package\nP c0 x").unwrap_err();
    assert!(format!("{err:#}").contains("line no.3"), "{err:#}");
}

#[cfg(test)]
fn wide(attrs: std::ops::RangeInclusive<usize>) -> String {
    let mut txt = String::from("F c0");
    for i in attrs.clone() {
        txt.push_str(&format!(" a{i}"));
    }
    for i in attrs {
        txt.push_str(&format!("\nF c{i}\nP c0 a{i} c{i}"));
    }
    txt
}

#[test]
fn deploys_widest_formation() -> Result<()> {
    assert_eq!(EDGES + 1, deploy(&wide(1..=EDGES))?.len());
    Ok(())
}

#[test]
fn rejects_too_wide_formation() {
    let err = deploy(&wide(1..=EDGES + 1)).unwrap_err();
    assert!(
        err.to_string()
            .contains(&format!("The cell c0 has {} edges", EDGES + 1)),
        "Unexpected error: {err}"
    );
}

#[test]
fn rejects_too_wide_merge() -> Result<()> {
    let left = deploy(&wide(1..=EDGES))?;
    let right = deploy("F c0 extra\nF c1\nP c0 extra c1")?;
    let err = merge(&left, &right).unwrap_err();
    assert!(
        err.to_string()
            .contains(&format!("The cell c0 would have {} edges", EDGES + 1)),
        "Unexpected error: {err}"
    );
    assert_eq!(EDGES + 1, merge(&left, &left)?.len());
    Ok(())
}
