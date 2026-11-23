// SPDX-FileCopyrightText: Copyright (c) 2022-2026 Yegor Bugayenko
// SPDX-License-Identifier: MIT

mod common;

use crate::common::runtime::load_runtime;
use anyhow::{Context, Result};
use glob::glob;
use log::debug;
use reo::org::eolang::register;
use reo::Universe;

fn all_apps() -> Result<Vec<String>> {
    let mut apps = Vec::new();
    for f in glob("eo-tests/**/*.eo")? {
        let path = f?;
        let app = path
            .strip_prefix("eo-tests")?
            .with_extension("")
            .to_str()
            .with_context(|| format!("Can't get str from '{}'", path.display()))?
            .replace('/', ".");
        apps.push(app);
    }
    Ok(apps)
}

#[test]
#[ignore]
fn deploys_and_runs_all_apps() -> Result<()> {
    let mut uni = Universe::from_graph(load_runtime()?)?;
    register(&mut uni);
    for app in all_apps()? {
        debug!("App: {app}");
        let c = uni.find(format!("Φ.{app}").as_str())?;
        debug!("{}", uni.slice(c)?.to_dot());
        let expected = uni.dataize(format!("Φ.{app}.expected").as_str())?;
        let actual = uni.dataize(format!("Φ.{app}").as_str())?;
        assert_eq!(expected, actual, "{app} failed");
        debug!("Evaluated {app} as {actual}!");
    }
    Ok(())
}
