use super::*;
fn bundle() -> Bundle {
    bundled()
        .unwrap()
        .into_iter()
        .find(|b| b.manifest.id == "product-docs")
        .unwrap()
}
fn root(temp: &tempfile::TempDir) -> &Utf8Path {
    Utf8Path::from_path(temp.path()).unwrap()
}
fn archive(entries: &BTreeMap<String, Vec<u8>>) -> Vec<u8> {
    let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    let mut builder = tar::Builder::new(encoder);
    for (path, bytes) in entries {
        let mut h = tar::Header::new_ustar();
        h.set_size(bytes.len() as u64);
        h.set_mode(0o644);
        h.set_cksum();
        builder.append_data(&mut h, path, bytes.as_slice()).unwrap();
    }
    builder.into_inner().unwrap().finish().unwrap()
}
#[test]
fn bundled_catalog_creates_three_valid_editable_sites() {
    let bundles = bundled().unwrap();
    assert_eq!(bundles.len(), 3);
    let temp = tempfile::tempdir().unwrap();
    for b in bundles {
        let name = b.manifest.id.clone();
        let target = root(&temp).join(&name);
        install::create(b, &target, Some("A: \"title\" # 日本"), &[]).unwrap();
        assert!(target.join("content").is_dir());
        assert!(target.join(".zel-template.json").is_file());
        assert!(!target.join("site").exists());
        let receipt: serde_json::Value =
            serde_json::from_slice(&fs::read(target.join(".zel-template.json")).unwrap()).unwrap();
        for (path, digest) in receipt["files"].as_object().unwrap() {
            assert_eq!(
                sha(&fs::read(target.join(path)).unwrap()),
                digest.as_str().unwrap()
            );
        }
        assert_eq!(receipt["template"], name);
    }
}
#[test]
fn substitution_is_typed_literal_single_pass_and_fails_on_unknown_tokens() {
    let value = "A: \"quoted\" # [x](javascript:bad) <script>{x}</script> {{zel.other|yaml}}";
    let values = BTreeMap::from([("title".into(), value.into())]);
    let yaml = install::substitute("name: {{zel.title|yaml}}", &values).unwrap();
    let parsed: serde_yaml_ng::Value = parse(yaml.as_bytes()).unwrap();
    assert_eq!(parsed["name"].as_str(), Some(value));
    let md = install::substitute("{{zel.title|markdown}}", &values).unwrap();
    assert!(!md.contains("<script>") && !md.contains("[x](") && !md.contains("{{zel."));
    for text in [
        "{{zel.unknown|yaml}}",
        "{{zel.title}}",
        "{{zel.title|raw}}",
        "{{zel.title|yaml",
    ] {
        assert!(install::substitute(text, &values).is_err());
    }
}
#[test]
fn wrong_variables_and_invalid_content_leave_destination_untouched() {
    let t = tempfile::tempdir().unwrap();
    let out = root(&t).join("out");
    for overrides in [
        vec!["missing=value".into()],
        vec!["title=one".into(), "title=two".into()],
        vec!["title=a\nb".into()],
        vec!["title".into()],
    ] {
        assert!(install::create(bundle(), &out, None, &overrides).is_err());
        assert!(!out.exists());
    }
    let mut b = bundle();
    b.entries.insert(
        "files/content/index.md".into(),
        b"[broken](missing.md)".to_vec(),
    );
    assert!(install::create(b, &out, None, &[]).is_err());
    assert!(!out.exists());
    assert_eq!(fs::read_dir(root(&t)).unwrap().count(), 0);
    fs::create_dir(&out).unwrap();
    fs::write(out.join(".authored"), "keep").unwrap();
    assert!(install::create(bundle(), &out, None, &[]).is_err());
    assert_eq!(fs::read_to_string(out.join(".authored")).unwrap(), "keep");
}
#[test]
fn empty_destination_can_be_published_without_overwrite_option() {
    let t = tempfile::tempdir().unwrap();
    let out = root(&t).join("out");
    fs::create_dir(&out).unwrap();
    install::create(bundle(), &out, None, &[]).unwrap();
    assert!(out.join("zel.yaml").is_file());
}
#[test]
fn archive_roundtrip_checks_checksum_trailer_and_duplicate_paths() {
    let b = bundle();
    let bytes = archive(&b.entries);
    let loaded = source::archive(&bytes, "fixture".into()).unwrap();
    assert_eq!(loaded.entries, b.entries);
    assert_eq!(loaded.digest, b.digest);
    let mut bad = bytes.clone();
    let end = bad.len();
    bad[end - 8] ^= 0xff;
    assert!(source::archive(&bad, "fixture".into()).is_err());
    let mut trailing = bytes.clone();
    trailing.extend_from_slice(b"hidden");
    assert!(source::archive(&trailing, "fixture".into()).is_err());
    assert!(source::archive(&bytes[..bytes.len() - 4], "fixture".into()).is_err());
}
#[test]
fn paths_assets_and_metadata_fail_before_writes() {
    for name in [
        "../outside",
        "/outside",
        "files/C:/bad",
        "files/a\\b",
        "files/.git/config",
        "files/CON.txt",
        "files/a.",
        "files/a ",
    ] {
        let mut entries = bundle().entries;
        entries.insert(name.into(), vec![]);
        assert!(
            Bundle::from_entries(entries, "fixture".into()).is_err(),
            "{name}"
        );
    }
    for name in [
        "files/CONTENT/index.md",
        "files/CONTENT/another.md",
        "files/content/index.md/child",
        "files/.zel-template.json",
        "files/.ZEL-TEMPLATE.JSON",
        "files/site/index.html",
    ] {
        let mut entries = bundle().entries;
        entries.insert(name.into(), vec![]);
        assert!(
            Bundle::from_entries(entries, "fixture".into()).is_err(),
            "{name}"
        );
    }
    for mutate in [0, 1, 2, 3] {
        let mut b = bundle();
        match mutate {
            0 => b.manifest.schema = 999,
            1 => b.manifest.minimum_cli = "999.0.0".into(),
            2 => b.manifest.assets.push(manifest::Asset {
                path: "content/missing.svg".into(),
                license: "test".into(),
            }),
            _ => b.manifest.substitute.push("missing.md".into()),
        }
        b.entries.insert(
            "template.yaml".into(),
            serde_yaml_ng::to_string(&b.manifest).unwrap().into_bytes(),
        );
        assert!(Bundle::from_entries(b.entries, "fixture".into()).is_err());
    }
}
#[test]
fn archives_reject_links_special_entries_and_raw_traversal() {
    for (name, kind) in [
        ("../outside", tar::EntryType::Regular),
        ("files/link", tar::EntryType::Symlink),
        ("files/hard", tar::EntryType::Link),
        ("files/fifo", tar::EntryType::Fifo),
        ("files/dir", tar::EntryType::Directory),
    ] {
        let gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        let mut tar = tar::Builder::new(gz);
        let mut h = tar::Header::new_ustar();
        h.as_mut_bytes()[..name.len()].copy_from_slice(name.as_bytes());
        h.set_entry_type(kind);
        h.set_size(0);
        h.set_mode(0o644);
        h.set_cksum();
        tar.append(&h, std::io::empty()).unwrap();
        let bytes = tar.into_inner().unwrap().finish().unwrap();
        assert!(source::archive(&bytes, "fixture".into()).is_err(), "{name}");
    }
}
#[cfg(unix)]
#[test]
fn local_templates_and_destinations_reject_links_without_following_them() {
    let t = tempfile::tempdir().unwrap();
    let source = root(&t).join("source");
    fs::create_dir(&source).unwrap();
    std::os::unix::fs::symlink("/nonexistent", source.join("template.yaml")).unwrap();
    assert!(source::local(&source).is_err());
    let out = root(&t).join("out");
    std::os::unix::fs::symlink(&source, &out).unwrap();
    assert!(install::destination(&out).is_err());
}
#[test]
fn remote_catalog_and_archive_cache_work_after_server_shutdown() {
    use std::net::TcpListener;
    let t = tempfile::tempdir().unwrap();
    let b = bundle();
    let bytes = archive(&b.entries);
    let server = TcpListener::bind("127.0.0.1:0").unwrap();
    let origin = format!("http://{}", server.local_addr().unwrap());
    let catalog = serde_json::to_vec(&source::Catalog {
        schema: 1,
        release: "v0.2.0-rc.5".into(),
        templates: vec![source::Entry {
            manifest: b.manifest.clone(),
            archive: source::Archive {
                url: format!("{origin}/template.tar.gz"),
                sha256: sha(&bytes),
            },
        }],
    })
    .unwrap();
    let worker = std::thread::spawn(move || {
        for body in [catalog, bytes] {
            let (mut client, _) = server.accept().unwrap();
            client
                .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                .unwrap();
            let mut request = Vec::new();
            let mut buf = [0; 1];
            while !request.ends_with(b"\r\n\r\n") {
                client.read_exact(&mut buf).unwrap();
                request.push(buf[0]);
                assert!(request.len() < 8192);
            }
            write!(
                client,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            )
            .unwrap();
            client.write_all(&body).unwrap();
        }
    });
    let mut options = TemplateSourceArgs {
        catalog: Some(format!("{origin}/catalog.json")),
        cache_dir: Some(root(&t).join("cache").into_string()),
        offline: false,
    };
    let loaded = resolve("product-docs@0.1.0", &options).unwrap();
    assert_eq!(loaded.entries, b.entries);
    worker.join().unwrap();
    options.offline = true;
    assert_eq!(
        resolve("product-docs", &options).unwrap().entries,
        b.entries
    );
    let cache = root(&t).join("cache");
    let file = fs::read_dir(cache)
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.extension().is_some_and(|s| s == "gz"))
        .unwrap();
    fs::write(file, "corrupt").unwrap();
    assert!(resolve("product-docs", &options).is_err());
}

#[test]
fn archives_refuse_duplicates_and_oversized_entries_before_destination_exists() {
    let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    let mut archive = tar::Builder::new(encoder);
    for _ in 0..2 {
        let mut h = tar::Header::new_ustar();
        h.set_size(0);
        h.set_mode(0o644);
        h.set_cksum();
        archive
            .append_data(&mut h, "files/repeated", std::io::empty())
            .unwrap();
    }
    let bytes = archive.into_inner().unwrap().finish().unwrap();
    assert!(source::archive(&bytes, "fixture".into()).is_err());
    let mut entries = bundle().entries;
    entries.insert("files/content/huge".into(), vec![0; FILE_LIMIT + 1]);
    assert!(Bundle::from_entries(entries, "fixture".into()).is_err());
}

#[test]
fn substitution_limits_are_enforced_while_expanding() {
    let values = BTreeMap::from([("title".into(), "x".repeat(4096))]);
    let input = "{{zel.title|markdown}}".repeat(1025);
    assert!(
        install::substitute(&input, &values)
            .unwrap_err()
            .to_string()
            .contains("size limit")
    );
}
