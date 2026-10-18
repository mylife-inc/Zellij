use super::*;
use clap::error::ErrorKind;
use std::collections::BTreeSet;

fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
    let matches = Cli::command_tree()
        .try_get_matches_from(std::iter::once("zel").chain(args.iter().copied()))?;
    Cli::from_matches(&matches)
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
    for name in ["bu", "bui", "ini", "n", "se", "na", "ni", "pub", "netlify"] {
        assert_eq!(
            parse(&[name]).unwrap_err().kind(),
            ErrorKind::InvalidSubcommand
        );
    }
    assert!(parse(&["ci"]).is_err());
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
    }
}
