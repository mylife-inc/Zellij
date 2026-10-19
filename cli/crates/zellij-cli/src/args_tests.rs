use super::*;
use clap::error::ErrorKind;
use std::collections::BTreeSet;

fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
    Cli::parse_tokens(std::iter::once("zel").chain(args.iter().copied()))
}

// Cover every registered shortcut with required-only and fully specified args.
// None of these parser tests invokes a runtime, browser, registry or workflow.
type Case = (
    &'static str,
    &'static str,
    &'static [&'static str],
    &'static [&'static str],
);
const CASES: &[Case] = &[
    (
        "na",
        "new app",
        &[],
        &[
            "web app",
            "--with-docs",
            "--engine",
            "engine/package",
            "--name",
            "My docs",
            "--port",
            "4100",
            "--features",
            "maths,diagrams",
        ],
    ),
    (
        "init",
        "new portable",
        &[],
        &[
            "Manual 日本",
            "--name",
            "My site",
            "--theme",
            "paper",
            "--minimal",
        ],
    ),
    (
        "nd",
        "new docs",
        &[],
        &[
            "manual",
            "--name",
            "Hosted",
            "--theme",
            "cupertino",
            "--guide",
            "--minimal",
            "--force",
        ],
    ),
    (
        "ns",
        "new section",
        &["hero"],
        &["hero", "--out", "hero.yaml"],
    ),
    (
        "np",
        "new page",
        &["home"],
        &["home", "--sections", "hero,prose", "--dir", "my content"],
    ),
    (
        "ng",
        "new guide",
        &["guide"],
        &["guide", "--title", "My guide", "--dir", "my content"],
    ),
    (
        "npr",
        "new project",
        &["product"],
        &[
            "product",
            "--product-lang",
            "rust",
            "--docs-dir",
            "docs/manual",
            "--product-dir",
            "src",
        ],
    ),
    ("ex", "explain", &["hero"], &["prose"]),
    ("sections", "list sections", &[], &[]),
    ("themes", "list themes", &[], &[]),
    ("formats", "list formats", &[], &[]),
    ("targets", "list targets", &[], &[]),
    ("ctx", "context", &[], &["reference.json"]),
    (
        "cv",
        "convert",
        &["source", "--out", "output"],
        &["source", "--out", "output", "--from", "mkdocs", "--dry-run"],
    ),
    (
        "mig",
        "migrate",
        &["source", "--out", "output"],
        &["source", "--out", "output"],
    ),
    (
        "c",
        "check",
        &[],
        &["manual", "--target", "portable", "--deep", "--strict"],
    ),
    (
        "b",
        "build",
        &[],
        &[
            "manual",
            "--target",
            "portable",
            "--out",
            "export",
            "--tag",
            "image:tag",
            "--name",
            "Site",
            "--theme",
            "paper",
        ],
    ),
    (
        "s",
        "serve",
        &[],
        &["manual", "--port", "8080", "--base-path", "/docs/"],
    ),
    (
        "v",
        "view",
        &[],
        &[
            "manual",
            "--out",
            "export",
            "--port",
            "0",
            "--name",
            "Site",
            "--theme",
            "paper",
            "--no-open",
            "--watch",
        ],
    ),
    (
        "ir",
        "image run",
        &[],
        &["registry.test/docs:tag", "--port", "8080"],
    ),
    (
        "ip",
        "image push",
        &["docs:tag"],
        &["docs:tag", "--registry", "registry.test"],
    ),
    (
        "si",
        "image static",
        &[],
        &["manual", "--tag", "registry.test/docs:static"],
    ),
    (
        "ni",
        "image next",
        &[],
        &["manual", "--tag", "registry.test/docs:next"],
    ),
    (
        "idf",
        "image dockerfile",
        &[],
        &["manual", "--kind", "static"],
    ),
    (
        "ci publish",
        "ci run",
        &[],
        &["--workflow", "other.yml", "--ref", "feature/docs"],
    ),
    (
        "pub",
        "publish",
        &[],
        &["manual", "--tag", "ghcr.io/acme/docs:v2", "--dry-run"],
    ),
    (
        "netlify",
        "deploy netlify",
        &[],
        &[
            "manual",
            "--site-id",
            "manual.netlify.app",
            "--prod",
            "--dry-run",
        ],
    ),
    ("comp", "completions", &["bash"], &["powershell"]),
];

fn invocation<'a>(route: &'a str, args: &[&'a str]) -> Vec<&'a str> {
    route
        .split_whitespace()
        .chain(args.iter().copied())
        .collect()
}

#[test]
fn shortcuts_preserve_defaults_options_and_global_flags() {
    for (short, full, minimal, explicit) in CASES {
        for args in [*minimal, *explicit] {
            let expected = parse(&invocation(full, args)).unwrap();
            assert_eq!(
                parse(&invocation(short, args)).unwrap(),
                expected,
                "{short}"
            );
            for prefix in [true, false] {
                let mut short_args = invocation(short, args);
                let mut full_args = invocation(full, args);
                if prefix {
                    short_args.insert(0, "--json");
                    full_args.insert(0, "--json");
                } else {
                    short_args.push("--json");
                    full_args.push("--json");
                }
                let actual = parse(&short_args).unwrap();
                assert!(actual.json);
                assert_eq!(actual, parse(&full_args).unwrap(), "{short}");
            }
        }
    }
}

#[test]
fn every_registered_shortcut_is_tested_and_names_do_not_collide() {
    Cli::command_tree().debug_assert();
    let mut actual: BTreeSet<String> = SHORTCUTS.iter().map(|(s, _)| (*s).into()).collect();
    fn aliases(command: &clap::Command, prefix: &str, result: &mut BTreeSet<String>) {
        for sub in command.get_subcommands() {
            for alias in sub.get_all_aliases() {
                assert!(
                    result.insert(format!("{prefix}{alias}")),
                    "duplicate shortcut"
                );
            }
            aliases(sub, &format!("{prefix}{} ", sub.get_name()), result);
        }
    }
    aliases(&Cli::command_tree(), "", &mut actual);
    actual.insert("netlify".into());
    assert_eq!(
        actual,
        CASES.iter().map(|(short, ..)| (*short).into()).collect()
    );
}

#[test]
fn help_links_the_short_and_descriptive_forms() {
    for (short, full, ..) in CASES {
        let short_help = parse(&invocation(short, &["--help"])).unwrap_err();
        let full_help = parse(&invocation(full, &["--help"])).unwrap_err();
        assert_eq!(short_help.kind(), ErrorKind::DisplayHelp);
        assert_eq!(full_help.kind(), ErrorKind::DisplayHelp);
        assert!(
            short_help.to_string().contains(full),
            "{short}: {short_help}"
        );
        assert!(
            full_help
                .to_string()
                .contains(short.split_whitespace().last().unwrap()),
            "{full}: {full_help}"
        );
    }
}

#[test]
fn invalid_arguments_keep_the_same_error_kind_and_status() {
    for (short, full, minimal, _) in CASES {
        for args in [&[][..], *minimal] {
            let mut invalid = args.to_vec();
            invalid.push("--not-a-zel-option");
            let short_error = parse(&invocation(short, &invalid)).unwrap_err();
            let full_error = parse(&invocation(full, &invalid)).unwrap_err();
            assert_eq!(short_error.kind(), full_error.kind(), "{short}");
            assert_eq!(short_error.exit_code(), full_error.exit_code(), "{short}");
        }
    }
    for (short, full, args) in [
        ("v", "view", vec!["--no-open", "--browser", "my-browser"]),
        ("s", "serve", vec!["--port", "65536"]),
        ("c", "check", vec!["--target", "other"]),
        ("comp", "completions", vec!["not-a-shell"]),
        ("idf", "image dockerfile", vec!["--kind", "other"]),
    ] {
        assert_eq!(
            parse(&invocation(short, &args)).unwrap_err().kind(),
            parse(&invocation(full, &args)).unwrap_err().kind()
        );
    }
}

#[test]
fn arguments_are_not_rewritten_and_unregistered_prefixes_are_rejected() {
    for (short, full) in [("b", "build"), ("init", "new portable"), ("ctx", "context")] {
        for value in ["b", "init", "--json", "日本 space"] {
            let expected = parse(&invocation(full, &["--", value])).unwrap();
            let actual = parse(&invocation(short, &["--", value])).unwrap();
            assert_eq!(actual, expected);
            assert!(!actual.json);
        }
    }
    for name in ["bu", "bui", "ini", "n", "se"] {
        assert_eq!(
            parse(&[name]).unwrap_err().kind(),
            ErrorKind::InvalidSubcommand
        );
    }
    assert!(parse(&["ci"]).is_ok());
    assert_eq!(
        parse(&["v", "--browser", "b"]).unwrap(),
        parse(&["view", "--browser", "b"]).unwrap()
    );
}

#[test]
fn completion_scripts_include_all_forms_and_options() {
    for shell in [
        clap_complete::Shell::Bash,
        clap_complete::Shell::Zsh,
        clap_complete::Shell::Fish,
        clap_complete::Shell::PowerShell,
        clap_complete::Shell::Elvish,
    ] {
        let mut script = Vec::new();
        clap_complete::generate(shell, &mut Cli::command_tree(), "zel", &mut script);
        let script = String::from_utf8(script).unwrap();
        for (short, full, ..) in CASES {
            for name in short.split_whitespace().chain(full.split_whitespace()) {
                assert!(script.contains(name), "{shell}: {name}");
            }
        }
        assert!(script.contains("no-open"), "{shell}");
        assert!(script.contains("base-path"), "{shell}");
        assert!(script.contains("cwd"), "{shell}");
        assert!(script.contains("site-root"), "{shell}");
        assert!(script.contains("config"), "{shell}");
    }
}

#[test]
fn global_selectors_normalize_all_site_commands_and_preserve_legacy_spelling() {
    for route in [
        "build",
        "b",
        "check",
        "c",
        "dev",
        "fmt",
        "watch",
        "serve",
        "s",
        "view",
        "v",
        "image static",
        "si",
        "image next",
        "ni",
        "image build",
        "image dockerfile",
        "idf",
        "publish",
        "pub",
        "deploy",
        "deploy netlify",
        "netlify",
        "eject",
    ] {
        let extra = if route == "eject" {
            vec!["--out", "wrapper"]
        } else {
            vec![]
        };
        let mut positional = invocation(route, &extra);
        // Bare deploy's first positional is the provider, not the site.
        if route == "deploy" {
            positional.push("netlify");
        }
        positional.push("Manual 日本");
        let expected = parse(&positional).unwrap();
        let actual_route = if route == "deploy" {
            "deploy netlify"
        } else {
            route
        };
        for prefix in [true, false] {
            let mut selected = invocation(actual_route, &extra);
            if prefix {
                selected.splice(0..0, ["--site-root", "Manual 日本"]);
            } else {
                selected.extend(["--site-root", "Manual 日本"]);
            }
            assert_eq!(
                parse(&selected).unwrap(),
                expected,
                "{route} prefix={prefix}"
            );
        }
    }
    for route in ["new page", "np", "new guide", "ng"] {
        assert_eq!(
            parse(&invocation(route, &["intro", "--site-root", "manual"])).unwrap(),
            parse(&invocation(route, &["intro", "--dir", "manual"])).unwrap()
        );
    }
    for route in ["ci", "ci init github"] {
        assert_eq!(
            parse(&invocation(route, &["--site-root", "manual"])).unwrap(),
            parse(&invocation(route, &["--content", "manual"])).unwrap()
        );
        let mut args = vec!["--site-root", "manual"];
        args.extend(route.split_whitespace());
        assert_eq!(
            parse(&args).unwrap(),
            parse(&invocation(route, &["--content", "manual"])).unwrap()
        );
    }
    for route in ["new app", "na", "features all", "image run", "ir"] {
        let mut args = vec!["--site-root", "manual"];
        args.extend(route.split_whitespace());
        assert_eq!(
            parse(&args).unwrap(),
            parse(&invocation(route, &["--site-root", "manual"])).unwrap()
        );
    }
}

#[test]
fn working_directory_is_global_without_reinterpreting_path_arguments() {
    for (short, full, minimal, _) in CASES {
        for route in [*short, *full] {
            let mut before = vec!["--cwd", "Repo 日本"];
            before.extend(invocation(route, minimal));
            let mut after = invocation(route, minimal);
            after.extend(["--cwd", "Repo 日本"]);
            let expected = parse(&before).unwrap();
            assert_eq!(expected.cwd.as_deref(), Some("Repo 日本"));
            assert_eq!(parse(&after).unwrap(), expected);
        }
    }
    assert_eq!(parse(&["build", "--", "--cwd"]).unwrap().cwd, None);
    for args in [["--cwd", "", "version"], ["build", "--site-root", ""]] {
        assert_eq!(parse(&args).unwrap_err().exit_code(), 2);
    }
}

#[test]
fn ambiguous_and_inapplicable_site_selections_are_rejected_during_parsing() {
    for args in [
        vec!["build", "manual"],
        vec!["c", "manual"],
        vec!["fmt", "manual"],
        vec!["ci", "manual"],
        vec!["ci", "--content", "manual"],
        vec!["ci", "init", "github", "--content", "manual"],
        vec!["new", "page", "page", "--dir", "manual"],
        vec!["ng", "guide", "--dir", "manual"],
        vec!["new", "app", "wrapper", "--with-docs"],
        vec!["eject", "--out", "wrapper", "--with-docs"],
        vec!["eject", "manual", "--out", "wrapper"],
        vec!["image", "run", "image:tag"],
        vec!["ip", "image:tag"],
        vec!["init"],
        vec!["nd"],
        vec!["npr", "product"],
        vec!["ns", "hero"],
        vec!["ci", "run"],
        vec!["context"],
        vec!["sections"],
        vec!["doctor"],
        vec!["version"],
        vec!["pull"],
        vec!["upgrade", "--check"],
        vec!["detect", "source"],
        vec!["convert", "source", "--out", "result"],
        vec!["mig", "source", "--out", "result"],
        vec!["ship", "source", "--out", "result", "--provider", "netlify"],
    ] {
        for prefix in [true, false] {
            let mut selected = args.clone();
            if prefix {
                selected.splice(0..0, ["--site-root", "manual"]);
            } else {
                selected.extend(["--site-root", "manual"]);
            }
            assert_eq!(parse(&selected).unwrap_err().exit_code(), 2, "{selected:?}");
        }
    }
}

#[test]
fn repeated_global_paths_fail_across_command_levels_without_rewriting_values() {
    for flag in ["--cwd", "--site-root", "--config"] {
        for other in ["a", "b"] {
            for args in [
                vec![flag, "a", "build", flag, other],
                vec![flag, "a", "image", flag, other, "static"],
                vec![flag, "a", "new", "app", flag, other],
            ] {
                assert_eq!(parse(&args).unwrap_err().exit_code(), 2, "{args:?}");
            }
        }
    }
    assert_eq!(
        parse(&["--site-root=a", "build", "--site-root=b"])
            .unwrap_err()
            .exit_code(),
        2
    );
    let value = parse(&["--cwd=--site-root", "build", "--name=--cwd", "--", "--cwd"]).unwrap();
    assert_eq!(value.cwd.as_deref(), Some("--site-root"));
    let Command::Build { dir, name, .. } = value.command else {
        panic!("expected build")
    };
    assert_eq!(dir, Some(SiteInput::from("--cwd")));
    assert_eq!(name.as_deref(), Some("--cwd"));
}

#[test]
fn explicit_config_keeps_its_identity_through_commands_and_aliases() {
    for (short, full, minimal, _) in CASES {
        for route in [*short, *full] {
            let mut args = invocation(route, minimal);
            args.extend(["--config", "Manual 日本/preview.yaml"]);
            let parsed = parse(&args);
            // Every existing-site operation accepts config; other argument roles
            // are deliberately rejected by the same normalization as site-root.
            let mut site_args = invocation(route, minimal);
            site_args.extend(["--site-root", "Manual 日本"]);
            assert_eq!(parsed.is_ok(), parse(&site_args).is_ok(), "{route}");
        }
    }
    let before = parse(&["--config", "preview.yaml", "b"]).unwrap();
    let after = parse(&["build", "--config=preview.yaml"]).unwrap();
    assert_eq!(before, after);
    let Command::Build { dir, .. } = before.command else {
        panic!("build")
    };
    assert_eq!(dir, Some(SiteInput::Config("preview.yaml".into())));
    for args in [
        vec!["--config=a.yaml", "build", "--config=b.yaml"],
        vec!["--config", "a.yaml", "build", "--site-root", "."],
        vec!["build", ".", "--config", "a.yaml"],
        vec!["ci", "--content", ".", "--config", "a.yaml"],
        vec![
            "ci",
            "init",
            "github",
            "--content",
            ".",
            "--config",
            "a.yaml",
        ],
        vec!["na", "app", "--with-docs", "--config", "a.yaml"],
        vec!["eject", "--out", "app", "--with-docs", "--config", "a.yaml"],
        vec!["ir", "example:latest", "--config", "a.yaml"],
        vec!["features", "none", "--config", ""],
    ] {
        assert_eq!(parse(&args).unwrap_err().exit_code(), 2, "{args:?}");
    }
}
