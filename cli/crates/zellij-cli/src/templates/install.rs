use super::*;

pub(super) fn destination(path: &Utf8Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(meta) => {
            let cwd = Utf8PathBuf::from_path_buf(std::env::current_dir()?)
                .map_err(|_| invalid("working directory is not UTF-8"))?
                .canonicalize_utf8()?;
            ensure(
                files::absolute_path(path)? != cwd,
                "choose a new/empty child folder, or run from its parent; replacing the current working directory would detach your shell",
            )?;
            ensure(
                meta.is_dir() && !zellij_core::site::is_link(&meta),
                "destination must be a new or empty directory, not a link",
            )?;
            ensure(
                fs::read_dir(path)?.next().is_none(),
                "destination is not empty; choose a new folder (existing files are never overwritten)",
            )
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}
fn markdown(value: &str) -> String {
    let mut out = String::new();
    for c in value.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '{' => out.push_str("&#123;"),
            '}' => out.push_str("&#125;"),
            c if c.is_ascii_punctuation() => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}
pub(super) fn substitute(input: &str, variables: &BTreeMap<String, String>) -> Result<String> {
    let mut rest = input;
    let mut output = String::new();
    while let Some(start) = rest.find("{{zel.") {
        ensure(
            output.len() + start <= FILE_LIMIT,
            "substitution exceeds file size limit",
        )?;
        output.push_str(&rest[..start]);
        rest = &rest[start + 6..];
        let end = rest
            .find("}}")
            .ok_or_else(|| invalid("unclosed template variable"))?;
        let (key, filter) = rest[..end].split_once('|').ok_or_else(|| {
            invalid("template variables require an explicit |yaml or |markdown filter")
        })?;
        let value = variables
            .get(key)
            .ok_or_else(|| invalid(format!("unknown template variable {key:?}")))?;
        let rendered = match filter {
            "yaml" => zellij_core::yaml::string(value),
            "markdown" => markdown(value),
            _ => return Err(invalid("unknown variable filter; use yaml or markdown")),
        };
        ensure(
            output.len() + rendered.len() <= FILE_LIMIT,
            "substitution exceeds file size limit",
        )?;
        output.push_str(&rendered);
        rest = &rest[end + 2..];
    }
    ensure(
        output.len() + rest.len() <= FILE_LIMIT,
        "substitution exceeds file size limit",
    )?;
    output.push_str(rest);
    Ok(output)
}
pub(super) fn create(
    bundle: Bundle,
    target: &Utf8Path,
    name: Option<&str>,
    overrides: &[String],
) -> Result<()> {
    destination(target)?;
    let mut variables = bundle.manifest.variables.clone();
    let mut assigned = BTreeSet::new();
    if let Some(name) = name {
        manifest::text(name)?;
        variables.insert("title".into(), name.into());
        assigned.insert("title".to_owned());
    }
    for assignment in overrides {
        let (key, value) = assignment
            .split_once('=')
            .ok_or_else(|| invalid("--set needs key=value"))?;
        ensure(
            variables.contains_key(key) && assigned.insert(key.into()),
            "unknown or duplicate template variable override",
        )?;
        manifest::text(value)?;
        variables.insert(key.into(), value.into());
    }
    let mut output = BTreeMap::new();
    let mut expanded_total = 0;
    for (path, bytes) in &bundle.entries {
        let Some(path) = path.strip_prefix("files/") else {
            continue;
        };
        let body = if bundle.manifest.substitute.iter().any(|p| p == path) {
            substitute(
                std::str::from_utf8(bytes)
                    .map_err(|_| invalid("substitution files must be UTF-8"))?,
                &variables,
            )?
            .into_bytes()
        } else {
            bytes.clone()
        };
        ensure(
            body.len() <= FILE_LIMIT,
            "substitution exceeds file size limit",
        )?;
        expanded_total += body.len();
        ensure(
            expanded_total <= TOTAL_LIMIT,
            "expanded template exceeds total size limit",
        )?;
        output.insert(path.to_owned(), body);
    }
    ensure(
        output.values().map(Vec::len).sum::<usize>() <= TOTAL_LIMIT,
        "expanded template exceeds total size limit",
    )?;
    // All substitution and path validation completes before parent directories are made.
    let absolute = files::absolute_path(target)?;
    let parent = absolute
        .parent()
        .ok_or_else(|| invalid("destination needs a parent"))?;
    fs::create_dir_all(parent)?;
    let stage = tempfile::Builder::new()
        .prefix(".zel-template-")
        .tempdir_in(parent)?;
    let root =
        Utf8Path::from_path(stage.path()).ok_or_else(|| invalid("temporary path is not UTF-8"))?;
    let entries: Vec<_> = output
        .iter()
        .map(|(p, b)| (Utf8Path::new(p), b.as_slice()))
        .collect();
    files::write_files(root, &entries, false)?;
    let site = zellij_core::site::Site::load(root)?;
    let expected = match bundle.manifest.renderer {
        Renderer::Portable => zellij_core::site::Renderer::Portable,
        Renderer::Hosted => zellij_core::site::Renderer::Hosted,
    };
    ensure(
        site.renderer == expected
            && site.content == root.join("content")
            && site.output == root.join("site"),
        "zel.yaml must match the manifest renderer and use content/ and site/ siblings",
    )?;
    if expected == zellij_core::site::Renderer::Portable {
        let report = zellij_docs::check(root, true);
        ensure(
            report.ok,
            &format!(
                "portable template validation failed: {}",
                serde_json::to_string(&report.diagnostics).unwrap_or_default()
            ),
        )?;
    } else {
        ensure(
            root.join("content/site.yaml").is_file() && root.join("content/pages.yaml").is_file(),
            "hosted template requires content/site.yaml and content/pages.yaml",
        )?;
        // Validate YAML syntax without launching Docker/Node. Deep hosted schema,
        // section/route validation remains `zel check --deep --strict` after creation.
        for (path, body) in &output {
            if path.ends_with(".yaml") || path.ends_with(".yml") {
                let _: serde_yaml_ng::Value = parse(body)?;
            }
        }
    }
    let receipt = serde_json::json!({"schema":1,"template":bundle.manifest.id,"version":bundle.manifest.version,
        "source":bundle.origin,"source_sha256":bundle.digest,"cli_version":env!("CARGO_PKG_VERSION"),"variables":variables,
        "files":output.iter().map(|(p,b)| (p.clone(),sha(b))).collect::<BTreeMap<_,_>>()});
    fs::write(
        root.join(".zel-template.json"),
        serde_json::to_string_pretty(&receipt).expect("receipt serialization") + "\n",
    )?;
    // Same-filesystem rename exposes the completed site. Recheck just before it;
    // removing a directory fails if another writer added any files meanwhile.
    destination(target)?;
    let existed = target.exists();
    if existed {
        fs::remove_dir(target)?;
    }
    if let Err(e) = fs::rename(root, &absolute) {
        if existed && !absolute.exists() {
            let _ = fs::create_dir(&absolute);
        }
        return Err(e.into());
    }
    Ok(())
}
