//! Real CLI execution: the short form must produce the same output, files and
//! failure status. These tests never publish, push, or contact a runtime.
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn run(root: &Path, args: &[&str], code: i32) -> Output {
    let result = Command::new(env!("CARGO_BIN_EXE_zel"))
        .current_dir(root)
        .args(args)
        .env("PATH", root.join("no-tools"))
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(code), "{args:?}: {result:?}");
    result
}

fn tree(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(base: &Path, dir: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(base, &path, files);
            } else {
                files.insert(
                    path.strip_prefix(base).unwrap().to_owned(),
                    fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut files = BTreeMap::new();
    visit(root, root, &mut files);
    files
}

#[test]
fn scaffold_aliases_create_the_same_files() {
    for (long, short) in [
        (
            vec!["new", "portable", "--minimal"],
            vec!["init", "--minimal"],
        ),
        (vec!["new", "docs", "--guide"], vec!["nd", "--guide"]),
        (
            vec!["new", "section", "hero", "--out", "hero.yaml"],
            vec!["ns", "hero", "--out", "hero.yaml"],
        ),
        (
            vec!["new", "page", "more", "--sections", "hero,markdown-prose"],
            vec!["np", "more", "--sections", "hero,markdown-prose"],
        ),
        (
            vec!["new", "guide", "manual", "--title", "User manual"],
            vec!["ng", "manual", "--title", "User manual"],
        ),
        (
            vec!["new", "project", "my-product", "--product-lang", "rust"],
            vec!["npr", "my-product", "--product-lang", "rust"],
        ),
    ] {
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        if matches!(short[0], "np" | "ng") {
            for root in [a.path(), b.path()] {
                run(root, &["new", "docs", "--minimal"], 0);
            }
        }
        run(a.path(), &long, 0);
        run(b.path(), &short, 0);
        assert_eq!(tree(a.path()), tree(b.path()), "{short:?}");
    }
}

#[test]
fn portable_build_check_and_failure_outputs_match() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    run(root, &["init", "--minimal"], 0);
    for (long, short, code) in [
        (vec!["build"], vec!["b"], 0),
        (
            vec!["check", "--json", "--strict"],
            vec!["--json", "c", "--strict"],
            0,
        ),
        (
            vec!["build", "--tag", "wrong"],
            vec!["b", "--tag", "wrong"],
            2,
        ),
        (
            vec!["serve", "missing", "--port", "0"],
            vec!["s", "missing", "--port", "0"],
            1,
        ),
        (vec!["view", "missing"], vec!["v", "missing"], 1),
    ] {
        let a = run(root, &long, code);
        let files = tree(root);
        let b = run(root, &short, code);
        assert_eq!(a.stdout, b.stdout, "{short:?}");
        assert_eq!(a.stderr, b.stderr, "{short:?}");
        assert_eq!(files, tree(root), "{short:?}");
    }
}

#[test]
fn inspection_aliases_keep_structured_output_and_generation() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    for (full, short) in [
        (vec!["list", "sections"], vec!["sections"]),
        (vec!["list", "themes"], vec!["themes"]),
        (vec!["list", "formats"], vec!["formats"]),
        (vec!["list", "targets"], vec!["targets"]),
        (vec!["context"], vec!["ctx"]),
        (vec!["explain", "hero"], vec!["ex", "hero"]),
    ] {
        for json in [false, true] {
            let mut a = full.clone();
            let mut b = short.clone();
            if json {
                a.insert(0, "--json");
                b.push("--json");
            }
            let a = run(root, &a, 0);
            let b = run(root, &b, 0);
            assert_eq!(a.stdout, b.stdout);
            assert_eq!(a.stderr, b.stderr);
            if json {
                serde_json::from_slice::<serde_json::Value>(&a.stdout).unwrap();
            }
        }
    }
    run(root, &["nd", "--minimal"], 0);
    let full = run(root, &["image", "dockerfile"], 0);
    let short = run(root, &["idf"], 0);
    assert_eq!(full.stdout, short.stdout);
    assert_eq!(full.stderr, short.stderr);
}

#[test]
fn unsupported_json_fails_before_any_alias_side_effects() {
    let tmp = tempfile::tempdir().unwrap();
    for args in [
        vec!["init"],
        vec!["nd"],
        vec!["ns", "hero"],
        vec!["ip", "test:latest"],
        vec!["ci", "run"],
        vec!["comp", "bash"],
    ] {
        let mut invocation = args;
        invocation.insert(0, "--json");
        let output = run(tmp.path(), &invocation, 2);
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("no command was run"));
        assert!(tree(tmp.path()).is_empty());
    }
}

#[test]
fn completions_are_pure_stdout_and_short_form_is_identical() {
    let tmp = tempfile::tempdir().unwrap();
    for shell in ["bash", "zsh", "fish", "powershell", "elvish"] {
        let full = run(tmp.path(), &["completions", shell], 0);
        let short = run(tmp.path(), &["comp", shell], 0);
        assert_eq!(full.stdout, short.stdout);
        assert!(full.stderr.is_empty() && short.stderr.is_empty());
        assert!(!full.stdout.is_empty());
    }
    assert!(tree(tmp.path()).is_empty());
}

#[cfg(unix)]
#[test]
fn bash_completes_options_for_canonical_and_short_commands() {
    let tmp = tempfile::tempdir().unwrap();
    let script = tmp.path().join("completion.bash");
    fs::write(&script, run(tmp.path(), &["comp", "bash"], 0).stdout).unwrap();
    let complete = |words: &[&str]| {
        let output = Command::new("bash").args(["--noprofile", "--norc", "-c",
            ". \"$1\"; shift; COMP_WORDS=(zel \"$@\" --); COMP_CWORD=$((${#COMP_WORDS[@]}-1)); COMP_LINE=\"${COMP_WORDS[*]}\"; COMP_POINT=${#COMP_LINE}; _zel zel -- \"${COMP_WORDS[COMP_CWORD-1]}\"; printf '%s\\n' \"${COMPREPLY[@]}\"", "--"])
            .arg(&script).args(words).output().unwrap();
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
        let mut options: Vec<_> = String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .map(str::to_owned)
            .collect();
        options.sort();
        options
    };
    for (full, short, flag) in [
        (vec!["build"], vec!["b"], "--out"),
        (vec!["new", "portable"], vec!["init"], "--minimal"),
        (vec!["view"], vec!["v"], "--no-open"),
        (vec!["image", "run"], vec!["ir"], "--port"),
        (vec!["ci", "run"], vec!["ci", "publish"], "--workflow"),
    ] {
        let full_options = complete(&full);
        assert!(
            full_options.iter().any(|item| item == flag),
            "{full:?}: {full_options:?}"
        );
        assert_eq!(full_options, complete(&short), "{short:?}");
    }
}

#[cfg(windows)]
#[test]
fn powershell_completes_options_for_both_forms() {
    let tmp = tempfile::tempdir().unwrap();
    let completion = tmp.path().join("zel-completion.ps1");
    fs::write(
        &completion,
        run(tmp.path(), &["comp", "powershell"], 0).stdout,
    )
    .unwrap();
    let check = tmp.path().join("check.ps1");
    fs::write(&check, r#"
param([string]$CompletionPath, [string]$ZelPath)
$ErrorActionPreference = 'Stop'
$resolved = (Get-Command zel -CommandType Application | Select-Object -First 1).Source
$expected = (Get-Item -LiteralPath $ZelPath).FullName
if ((Get-Item -LiteralPath $resolved).FullName -ne $expected) {
    throw "Completion must resolve $expected; resolved $resolved"
}
. $CompletionPath
foreach ($pair in @(
    @('zel build --', 'zel b --', '--out'),
    @('zel new portable --', 'zel init --', '--minimal'),
    @('zel view --', 'zel v --', '--no-open'),
    @('zel image run --', 'zel ir --', '--port'),
    @('zel ci run --', 'zel ci publish --', '--workflow')
)) {
    $Error.Clear()
    $full = @((TabExpansion2 -inputScript $pair[0] -cursorColumn $pair[0].Length).CompletionMatches.CompletionText | Sort-Object)
    $short = @((TabExpansion2 -inputScript $pair[1] -cursorColumn $pair[1].Length).CompletionMatches.CompletionText | Sort-Object)
    if ($full -notcontains $pair[2]) {
        $details = $Error | Format-List * -Force | Out-String
        throw "PowerShell $($PSVersionTable.PSVersion): missing $($pair[2]) for $($pair[0]); got: $full; errors: $details"
    }
    if (($full -join "`n") -cne ($short -join "`n")) { throw "Different options for $($pair[1])" }
}
"#).unwrap();
    // PowerShell only invokes native argument completers for discoverable
    // executables. Cargo's target directory is not on a runner's PATH.
    let binary = Path::new(env!("CARGO_BIN_EXE_zel"));
    let mut search = vec![binary.parent().unwrap().to_path_buf()];
    search.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    let result = Command::new("powershell.exe")
        .env("PATH", std::env::join_paths(search).unwrap())
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
        ])
        .arg(check)
        .arg("-CompletionPath")
        .arg(completion)
        .arg("-ZelPath")
        .arg(binary)
        .output()
        .unwrap();
    assert!(result.status.success(), "{result:?}");
    assert!(result.stderr.is_empty(), "{result:?}");
}

#[cfg(unix)]
#[test]
fn ci_dispatch_spellings_invoke_the_same_workflow_without_network() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = tempfile::tempdir().unwrap();
    let bin = tmp.path().join("no-tools");
    fs::create_dir(&bin).unwrap();
    let gh = bin.join("gh");
    for code in [0, 1] {
        fs::write(&gh, format!("#!/bin/sh\n[ \"$1\" = --version ] && exit 0\nprintf '%s\\n' \"$@\" > calls\nexit {code}\n")).unwrap();
        fs::set_permissions(&gh, fs::Permissions::from_mode(0o755)).unwrap();
        let full = run(
            tmp.path(),
            &[
                "ci",
                "run",
                "--workflow",
                "other.yml",
                "--ref",
                "feature/docs",
            ],
            code,
        );
        let first = fs::read(tmp.path().join("calls")).unwrap();
        let alias = run(
            tmp.path(),
            &[
                "ci",
                "publish",
                "--workflow",
                "other.yml",
                "--ref",
                "feature/docs",
            ],
            code,
        );
        assert_eq!(first, b"workflow\nrun\nother.yml\n--ref\nfeature/docs\n");
        assert_eq!(first, fs::read(tmp.path().join("calls")).unwrap());
        assert_eq!(full.stdout, alias.stdout);
        assert_eq!(full.stderr, alias.stderr);
    }
}
