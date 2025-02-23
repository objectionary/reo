// SPDX-FileCopyrightText: Copyright (c) 2022-2025 Yegor Bugayenko
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
        let p = f?;
        let path = p.as_path();
        let app = path
            .to_str()
            .context(format!("Can't get str from '{}'", path.display()))?
            .splitn(2, "/")
            .nth(1)
            .context(format!("Can't take path from '{}'", path.display()))?
            .split(".")
            .collect::<Vec<&str>>()
            .split_last()
            .context(format!("Can't take split_last from '{}'", path.display()))?
            .1
            .join(".")
            .replace("/", ".");
        apps.push(app.to_string());
    }
    Ok(apps)
}

#[test]
#[ignore]
fn deploys_and_runs_all_apps() -> Result<()> {
    let mut uni = Universe::from_graph(load_runtime()?);
    register(&mut uni);
    for app in all_apps()? {
        debug!("App: {app}");
        debug!(
            "{}",
            uni.slice(format!("ν0.{}", app).as_str()).unwrap().to_dot()
        );
        let expected = uni.dataize(format!("Φ.{}.expected", app).as_str()).unwrap();
        let actual = uni.dataize(format!("Φ.{}", app).as_str()).unwrap();
        assert_eq!(expected, actual, "{} failed", app);
        debug!("Evaluated {app} as {actual}!");
    }
    Ok(())
}
