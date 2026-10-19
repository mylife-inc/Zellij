//! Native feature selection: portable settings or an editable wrapper manifest.
mod yaml;
use camino::{Utf8Path, Utf8PathBuf};
use serde_json::Value;
use std::{fs, io::Write};
use zellij_core::{
    Error, Result, files,
    site::{Renderer, Site},
};

pub(crate) fn run(options: &crate::args::FeaturesArgs) -> Result<()> {
    let selection = &options.selection;
    if selection.len() > 1
        && selection
            .iter()
            .any(|s| matches!(s.as_str(), "all" | "none"))
    {
        return Err(Error::InvalidArgument(
            "all and none must be used alone".into(),
        ));
    }
    let maths = selection
        .iter()
        .any(|s| matches!(s.as_str(), "all" | "maths"));
    let diagrams = selection
        .iter()
        .any(|s| matches!(s.as_str(), "all" | "diagrams"));
    let cwd = Utf8PathBuf::from_path_buf(std::env::current_dir()?)
        .map_err(|_| Error::InvalidArgument("working directory is not UTF-8".into()))?;
    let root = if let Some(root) = &options.site_root {
        files::absolute_path(Utf8Path::new(root))?
    } else if fs::symlink_metadata(cwd.join("zellij.config.yaml")).is_ok() {
        cwd
    } else {
        crate::select_site(None)?
    };
    if fs::symlink_metadata(root.join("zellij.config.yaml")).is_ok() {
        wrapper(&root, maths, diagrams)?;
        println!(
            "Wrapper feature selection saved at {root}/package.json.\nRun `npm install` in the wrapper to reconcile its lockfile and installed packages.\nRebuild or restart the app to apply rendering changes. Disabled packages may remain if another dependency needs them."
        );
    } else {
        let site = Site::load(&root)?;
        if site.renderer != Renderer::Portable {
            return Err(Error::InvalidArgument("hosted content has no npm manifest; pass --site-root <wrapper> after `zel new app` or `zel eject`, or select an engine image variant for container builds".into()));
        }
        zellij_docs::validate_settings(site.portable)?;
        let path = files::contained_path(&root, Utf8Path::new("zel.yaml"))?;
        if !path.is_file() {
            return Err(Error::InvalidArgument("portable features require an existing zel.yaml; create a site with `zel init` first".into()));
        }
        let original = fs::read_to_string(&path)?;
        let changed = yaml::select(&original, maths, diagrams)?;
        replace(&root, "zel.yaml", &original, &changed)?;
        println!(
            "Portable settings saved: markdown.math={maths}, markdown.mermaid={diagrams}.\nRebuild with `zel build` to apply them. No tools or dependencies were installed."
        );
    }
    Ok(())
}

fn wrapper(root: &Utf8Path, maths: bool, diagrams: bool) -> Result<()> {
    let config = files::contained_path(root, Utf8Path::new("zellij.config.yaml"))?;
    let config: serde_yaml_ng::Value = serde_yaml_ng::from_str(&fs::read_to_string(config)?)
        .map_err(|e| Error::InvalidArgument(format!("invalid wrapper config: {e}")))?;
    if config["version"].as_u64() != Some(1) || config["content"].as_str().is_none() {
        return Err(Error::InvalidArgument(
            "expected a version: 1 wrapper config with a content path".into(),
        ));
    }
    let path = files::contained_path(root, Utf8Path::new("package.json"))?;
    let original = fs::read_to_string(path)?;
    let mut manifest: Value = serde_json::from_str(&original)
        .map_err(|e| Error::InvalidArgument(format!("invalid wrapper package.json: {e}")))?;
    let dependency = manifest["dependencies"]["@shebka/zellij"]
        .as_str()
        .and_then(|s| s.strip_prefix("file:"))
        .ok_or_else(|| {
            Error::InvalidArgument(
                "expected a Zellij wrapper with a local file: engine dependency".into(),
            )
        })?;
    let engine = files::absolute_path(&root.join(dependency))?;
    crate::app::validate_engine(&engine)?;
    let template = zellij_scaffold::app::template();
    let features = template["features"].as_object().unwrap();
    // Other package fields/dependencies are retained. Like the engine command,
    // these named feature packages are replaced as a complete selection.
    for field in [
        "dependencies",
        "devDependencies",
        "optionalDependencies",
        "peerDependencies",
    ] {
        if let Some(value) = manifest.get(field) {
            let deps = value
                .as_object()
                .ok_or_else(|| Error::InvalidArgument(format!("{field} must be an object")))?;
            if field != "dependencies"
                && features.values().any(|f| {
                    f.as_object()
                        .unwrap()
                        .keys()
                        .any(|name| deps.contains_key(name))
                })
            {
                return Err(Error::InvalidArgument(format!(
                    "feature packages also occur in {field}; reconcile them manually before changing the selection"
                )));
            }
        }
    }
    let deps = manifest["dependencies"].as_object_mut().unwrap();
    for (name, packages) in features {
        for (package, version) in packages.as_object().unwrap() {
            deps.remove(package);
            if (name == "maths" && maths) || (name == "diagrams" && diagrams) {
                deps.insert(package.clone(), version.clone());
            }
        }
    }
    if manifest.get("zellij").is_none() {
        manifest["zellij"] = serde_json::json!({});
    }
    let settings = manifest["zellij"]
        .as_object_mut()
        .ok_or_else(|| Error::InvalidArgument("package.json zellij must be an object".into()))?;
    settings.insert(
        "features".into(),
        Value::Array(
            template["featureNames"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|name| {
                    (name.as_str() == Some("maths") && maths)
                        || (name.as_str() == Some("diagrams") && diagrams)
                })
                .cloned()
                .collect(),
        ),
    );
    if manifest.get("overrides").is_none() {
        manifest["overrides"] = serde_json::json!({});
    }
    let overrides = manifest["overrides"]
        .as_object_mut()
        .ok_or_else(|| Error::InvalidArgument("overrides must be an object".into()))?;
    for (name, value) in template["manifest"]["overrides"].as_object().unwrap() {
        // Do not silently replace unrelated nested user overrides.
        match overrides.get_mut(name) {
            Some(Value::Object(existing)) if value.is_object() => {
                existing.extend(value.as_object().unwrap().clone());
            }
            _ => {
                overrides.insert(name.clone(), value.clone());
            }
        }
    }
    replace(
        root,
        "package.json",
        &original,
        &format!("{}\n", serde_json::to_string_pretty(&manifest).unwrap()),
    )
}

fn replace(root: &Utf8Path, name: &str, original: &str, changed: &str) -> Result<()> {
    if original == changed {
        return Ok(());
    }
    let path = files::contained_path(root, Utf8Path::new(name))?;
    let mut stage = tempfile::NamedTempFile::new_in(root)?;
    stage
        .as_file()
        .set_permissions(fs::metadata(&path)?.permissions())?;
    stage.write_all(changed.as_bytes())?;
    stage.flush()?;
    files::contained_path(root, Utf8Path::new(name))?;
    if fs::read_to_string(&path)? != original {
        return Err(files::unsafe_path(
            &path,
            "file changed while selecting features; retry after saving",
        ));
    }
    stage.persist(path).map_err(|e| e.error)?;
    Ok(())
}
