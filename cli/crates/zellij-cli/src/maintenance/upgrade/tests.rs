use super::*;

#[test]
fn release_selection_uses_only_our_stable_semantic_versions() {
    let releases: Vec<Release> = serde_json::from_value(serde_json::json!([
        {"tag_name":"other-v99.0.0", "draft":false, "prerelease":false},
        {"tag_name":"zellij-v0.9.0", "draft":false, "prerelease":false},
        {"tag_name":"zellij-v0.10.0", "draft":false, "prerelease":false},
        {"tag_name":"zellij-v1.0.0", "draft":true, "prerelease":false},
        {"tag_name":"zellij-v2.0.0", "draft":false, "prerelease":true},
        {"tag_name":"zellij-v3.0.0-rc.1", "draft":false, "prerelease":false},
        {"tag_name":"zellij-v4.0.0+metadata", "draft":false, "prerelease":false}
    ]))
    .unwrap();
    assert_eq!(stable(&releases), Some(Version::new(0, 10, 0)));
    for invalid in [
        "latest",
        "../0.2.0",
        "1.0",
        "01.2.3",
        "1.0.0+extra",
        "1.0.0-01",
    ] {
        assert!(parse_version(invalid).is_err());
    }
    assert_eq!(
        parse_version("v0.2.0-rc.1").unwrap().to_string(),
        "0.2.0-rc.1"
    );
}

#[test]
fn checksums_match_exact_names_and_reject_ambiguous_or_malformed_entries() {
    let hash = "a".repeat(64);
    for name in ["pkg", "./pkg", "*pkg", "*./pkg"] {
        assert_eq!(
            checksum(&format!("{hash}  {name}\r\n"), "pkg").unwrap(),
            Some(hash.clone())
        );
    }
    assert_eq!(
        checksum(&format!("{hash}  pkg-extra"), "pkg").unwrap(),
        None
    );
    for text in [
        "bad pkg".to_owned(),
        format!("{hash} pkg extra"),
        format!("{hash} pkg\n{} pkg", "b".repeat(64)),
    ] {
        assert!(checksum(&text, "pkg").is_err());
    }
}

#[test]
fn mirrors_and_limits_fail_closed() {
    for value in [
        "file:///tmp",
        "http://example.com/assets",
        "https://user:secret@example.com",
        "https://example.com?a=1",
        "https://example.com/#x",
        "http://127.0.0.1.example.com",
    ] {
        assert!(mirror(value).is_err(), "{value}");
    }
    for value in [
        "https://example.com/assets",
        "http://localhost:1234",
        "http://[::1]:1234",
        "http://127.0.0.1:1234/assets",
    ] {
        assert!(mirror(value).is_ok(), "{value}");
    }
    let stop = AtomicBool::new(false);
    assert!(copy_limited(&b"12345"[..], Vec::new(), 4, &stop).is_err());
    stop.store(true, Ordering::Relaxed);
    assert!(copy_limited(&b"1"[..], Vec::new(), 4, &stop).is_err());
}

#[test]
fn installation_lock_serializes_updates_and_preserves_unrelated_files() {
    let root = tempfile::tempdir().unwrap();
    let exe = root.path().join("zel");
    let lock = installation_lock(&exe).unwrap();
    assert!(installation_lock(&exe).is_err());
    drop(lock);
    drop(installation_lock(&exe).unwrap());
    fs::write(root.path().join(".zel.upgrade.lock"), "authored").unwrap();
    assert!(installation_lock(&exe).is_err());
    assert_eq!(
        fs::read_to_string(root.path().join(".zel.upgrade.lock")).unwrap(),
        "authored"
    );
}

#[test]
fn archive_extraction_rejects_links_extra_entries_and_wrong_names() {
    let root = tempfile::tempdir().unwrap();
    let stop = AtomicBool::new(false);
    for (name, symlink, extra, valid) in [
        ("zel", false, false, true),
        ("zel", true, false, false),
        ("elsewhere", false, false, false),
        ("zel", false, true, false),
    ] {
        let path = root.path().join("package.tar.gz");
        let gzip = flate2::write::GzEncoder::new(
            File::create(&path).unwrap(),
            flate2::Compression::default(),
        );
        let mut tar = tar::Builder::new(gzip);
        let mut header = tar::Header::new_gnu();
        header.set_mode(0o755);
        header.set_size(3);
        if symlink {
            header.set_entry_type(tar::EntryType::Symlink);
            header.set_link_name("../outside").unwrap();
        }
        header.set_cksum();
        tar.append_data(&mut header, name, &b"abc"[..]).unwrap();
        if extra {
            tar.append_data(&mut header, "extra", &b"abc"[..]).unwrap();
        }
        tar.into_inner().unwrap().finish().unwrap();
        let candidate = root.path().join("candidate");
        assert_eq!(
            archive::extract(&path, &candidate, false, &stop).is_ok(),
            valid
        );
        fs::remove_file(candidate).unwrap();
    }
    for (name, mode, extra, valid) in [
        ("zel.exe", 0o755, false, true),
        ("../zel.exe", 0o755, false, false),
        ("zel.exe", 0o755, true, false),
    ] {
        let path = root.path().join("package.zip");
        let mut zip = zip::ZipWriter::new(File::create(&path).unwrap());
        let options = zip::write::SimpleFileOptions::default().unix_permissions(mode);
        zip.start_file(name, options).unwrap();
        zip.write_all(b"abc").unwrap();
        if extra {
            zip.start_file("extra", options).unwrap();
            zip.write_all(b"abc").unwrap();
        }
        zip.finish().unwrap();
        let candidate = root.path().join("candidate");
        assert_eq!(
            archive::extract(&path, &candidate, true, &stop).is_ok(),
            valid
        );
        fs::remove_file(candidate).unwrap();
    }
    assert!(!root.path().join("outside").exists());
}

#[test]
fn executable_architecture_is_checked_before_running_it() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("candidate");
    for (target, arm) in [
        ("x86_64-unknown-linux-gnu", false),
        ("aarch64-unknown-linux-gnu", true),
        ("x86_64-apple-darwin", false),
        ("aarch64-apple-darwin", true),
        ("x86_64-pc-windows-msvc", false),
        ("aarch64-pc-windows-msvc", true),
    ] {
        let mut bytes = vec![0; 128];
        if target.contains("linux") {
            bytes[..6].copy_from_slice(b"\x7fELF\x02\x01");
            bytes[18..20].copy_from_slice(&(if arm { 183u16 } else { 62u16 }).to_le_bytes());
        } else if target.contains("darwin") {
            bytes[..4].copy_from_slice(&[0xcf, 0xfa, 0xed, 0xfe]);
            bytes[4..8]
                .copy_from_slice(&(if arm { 0x100000cu32 } else { 0x1000007u32 }).to_le_bytes());
        } else {
            bytes[..2].copy_from_slice(b"MZ");
            bytes[60..64].copy_from_slice(&64u32.to_le_bytes());
            bytes[64..68].copy_from_slice(b"PE\0\0");
            bytes[68..70].copy_from_slice(&(if arm { 0xaa64u16 } else { 0x8664u16 }).to_le_bytes());
        }
        fs::write(&path, &bytes).unwrap();
        archive::verify_target(&path, target).unwrap();
        let other = target.replace(
            if arm { "aarch64" } else { "x86_64" },
            if arm { "x86_64" } else { "aarch64" },
        );
        assert!(archive::verify_target(&path, &other).is_err());
    }
    fs::write(&path, b"#!/bin/sh\necho zel 0.2.0").unwrap();
    assert!(archive::verify_target(&path, crate::TARGET).is_err());
}
