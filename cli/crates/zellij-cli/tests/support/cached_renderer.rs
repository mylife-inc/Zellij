use serde_json::json;
use sha2::{Digest, Sha256};
use std::{fs, os::unix::fs::PermissionsExt, path::Path, process::Command};

pub fn install(root: &Path) {
    let os = if cfg!(target_os = "macos") {
        "darwin"
    } else {
        "linux"
    };
    let arch = if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "x64"
    };
    let platform = format!("{os}-{arch}");
    let version = env!("CARGO_PKG_VERSION");
    let cached = root.join(format!("resources/cache/engines/{version}-{platform}"));
    fs::create_dir_all(cached.join("engine")).unwrap();
    let script = r#"#!/bin/sh
if [ -n "${EXPECTED_CONTENT:-}" ]; then
  test -f "$EXPECTED_CONTENT/site.yaml" || exit 99
  test -f "$EXPECTED_CONTENT/01-guide/index.mdx" || exit 99
fi
printf 'check\n' >> "$FAKE_CALLS"
/bin/cp "$2" "$FAKE_REQUEST"
/bin/cat "$FAKE_REPORT" > check-report.json
echo 'internal fixture stderr must not pollute JSON' >&2
exit "${FAKE_EXIT:-0}"
"#;
    let files = [
        ("node", script),
        ("engine/engine.mjs", "fixture"),
        ("engine/runtime.js", "fixture"),
        ("engine/styles.css", "fixture"),
    ];
    let mut hashes = serde_json::Map::new();
    for (name, data) in files {
        fs::write(cached.join(name), data).unwrap();
        hashes.insert(name.into(), json!(format!("{:x}", Sha256::digest(data))));
    }
    fs::set_permissions(cached.join("node"), fs::Permissions::from_mode(0o755)).unwrap();
    fs::write(
        cached.join("manifest.json"),
        json!({"format":1,"version":version,
        "platform":platform,"node_version":"v24.0.0","files":hashes})
        .to_string(),
    )
    .unwrap();
}

pub fn configure(cmd: &mut Command, root: &Path) {
    cmd.env("ZELLIJ_HOME", root.join("resources"))
        .env_remove("ZEL_ENGINE_CACHE")
        .env("FAKE_REPORT", root.join("engine-report"))
        .env("FAKE_REQUEST", root.join("request"))
        .env("FAKE_CALLS", root.join("calls"));
}
