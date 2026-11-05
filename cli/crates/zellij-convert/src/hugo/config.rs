use super::data::*;
use camino::{Utf8Path, Utf8PathBuf};
use serde_yaml_ng::Value;
use zellij_core::{Result, files};

pub(super) fn markers(source: &Utf8Path) -> Vec<Utf8PathBuf> {
    [
        "hugo.toml",
        "hugo.yaml",
        "hugo.yml",
        "hugo.json",
        "config.toml",
        "config.yaml",
        "config.yml",
        "config.json",
    ]
    .into_iter()
    .map(Utf8PathBuf::from)
    .filter(|p| std::fs::symlink_metadata(source.join(p)).is_ok())
    .collect()
}
pub(super) fn read(source: &Utf8Path) -> Result<Value> {
    let configs = markers(source);
    if configs.len() != 1 {
        return Err(invalid(
            "expected exactly one hugo/config TOML, YAML or JSON file; consolidate configuration in a source copy",
        ));
    }
    let config_dir = files::contained_path(source, Utf8Path::new("config"))?;
    if config_dir.exists()
        && (!config_dir.is_dir() || std::fs::read_dir(&config_dir)?.next().is_some())
    {
        return Err(invalid(
            "split/environment configuration requires consolidation in a source copy",
        ));
    }
    let config = parse(
        &std::fs::read_to_string(files::contained_path(source, &configs[0])?)?,
        configs[0].extension().unwrap(),
    )?;
    for key in [
        "module",
        "languages",
        "roles",
        "versions",
        "cascade",
        "permalinks",
        "frontmatter",
        "ignorefiles",
        "ignorelogs",
        "ignorevendorpaths",
        "outputs",
        "outputformats",
        "mediatypes",
        "contenttypes",
        "segments",
        "rendersegments",
        "build",
        "menus",
        "menu",
        "sectionpagesmenu",
        "disablekinds",
    ] {
        if active(&config, key) {
            return Err(invalid(format!(
                "{key} requires manual migration before importing"
            )));
        }
    }
    for key in [
        "defaultcontentlanguageinsubdir",
        "defaultcontentroleinsubdir",
        "defaultcontentversioninsubdir",
        "canonifyurls",
        "relativeurls",
        "removepathaccents",
    ] {
        if boolean(&config, key)? {
            return Err(invalid(format!("{key} is not supported")));
        }
    }
    if active(&config, "uglyurls") && config["uglyurls"] != Value::Bool(false) {
        return Err(invalid(
            "uglyURLs requires manual migration; this importer supports directory URLs",
        ));
    }
    if string(&config, "timezone")?.is_some_and(|v| !matches!(v, "" | "UTC")) {
        return Err(invalid(
            "timeZone must be UTC; normalize dated metadata in a source copy",
        ));
    }
    for key in [
        "builddrafts",
        "buildfuture",
        "buildexpired",
        "disablepathtolower",
    ] {
        boolean(&config, key)?;
    }
    Ok(config)
}
