use std::{fs, path::Path, process::Command};

#[test]
fn rich_doctor_honors_resource_home_without_creating_it() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir(temp.path().join("content")).unwrap();
    fs::write(
        temp.path().join("zel.yaml"),
        "version: 1\nrenderer: hosted\n",
    )
    .unwrap();
    let home = temp.path().join("Resources café 日本");
    let command = || {
        let mut command = Command::new(env!("CARGO_BIN_EXE_zel"));
        command
            .current_dir(temp.path())
            .args(["doctor", "--json"])
            .env("ZELLIJ_HOME", &home)
            .env_remove("ZEL_ENGINE_CACHE");
        command
    };
    let output = command().output().unwrap();
    assert!(output.status.success(), "{output:?}");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let cache = Path::new(report["rich_engine"]["cache"].as_str().unwrap());
    assert_eq!(cache.parent().unwrap(), home.join("cache").join("engines"));
    assert!(
        !home.exists(),
        "doctor must not create the resource directory"
    );
    let explicit = temp.path().join("separate-engine-cache");
    let output = command()
        .env("ZELLIJ_HOME", "relative-is-invalid")
        .env("ZEL_ENGINE_CACHE", &explicit)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        Path::new(report["rich_engine"]["cache"].as_str().unwrap())
            .parent()
            .unwrap(),
        explicit
    );
    let output = command()
        .env("ZELLIJ_HOME", "relative-is-invalid")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("ZELLIJ_HOME must be an absolute path")
    );
    assert!(!temp.path().join("relative-is-invalid").exists());
}
