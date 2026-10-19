use super::*;

const TARGETS: &[&str] = &[
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc",
    "aarch64-pc-windows-msvc",
];

pub(super) fn supported(target: &str) -> Result<()> {
    if TARGETS.contains(&target) {
        Ok(())
    } else {
        Err(failed(
            "this build target has no native release archive; update using the method that installed it",
        ))
    }
}

pub(super) fn extract(
    archive: &Path,
    candidate: &Path,
    windows: bool,
    stop: &AtomicBool,
) -> Result<()> {
    let mut output = File::create_new(candidate)?;
    let file = File::open(archive)?;
    if windows {
        let mut zip = zip::ZipArchive::new(file).map_err(|_| failed("invalid ZIP archive"))?;
        if zip.len() != 1 {
            return Err(failed(
                "archive must contain exactly one regular zel.exe at its root",
            ));
        }
        let mut entry = zip.by_index(0).map_err(|_| failed("invalid ZIP entry"))?;
        let kind = entry.unix_mode().unwrap_or(0) & 0o170000;
        if entry.name() != "zel.exe"
            || !entry.is_file()
            || !matches!(kind, 0 | 0o100000)
            || entry.size() > BINARY_LIMIT
        {
            return Err(failed(
                "archive must contain exactly one regular zel.exe at its root",
            ));
        }
        copy_limited(&mut entry, &mut output, BINARY_LIMIT, stop)?;
    } else {
        let mut tar = tar::Archive::new(flate2::read::GzDecoder::new(file).take(BINARY_LIMIT + 1));
        let mut entries = tar.entries().map_err(|_| failed("invalid tar archive"))?;
        let mut entry = entries
            .next()
            .ok_or_else(|| failed("empty release archive"))?
            .map_err(|_| failed("invalid tar entry"))?;
        if entry.path_bytes().as_ref() != b"zel"
            || !entry.header().entry_type().is_file()
            || entry.size() > BINARY_LIMIT
        {
            return Err(failed(
                "archive must contain exactly one regular zel at its root",
            ));
        }
        copy_limited(&mut entry, &mut output, BINARY_LIMIT, stop)?;
        drop(entry);
        if entries.next().is_some() {
            return Err(failed("archive contains extra entries"));
        }
        // Drain the bounded gzip stream to verify its trailer/checksum too.
        copy_limited(tar.into_inner(), std::io::sink(), BINARY_LIMIT, stop)?;
    }
    output.sync_all()?;
    drop(output);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(candidate, fs::Permissions::from_mode(0o755))?;
    }
    Ok(())
}

pub(super) fn verify_target(path: &Path, target: &str) -> Result<()> {
    supported(target)?;
    let mut header = Vec::new();
    File::open(path)?.take(65536).read_to_end(&mut header)?;
    let arm = target.starts_with("aarch64-");
    let valid = if target.contains("linux") {
        header.get(..6) == Some(&b"\x7fELF\x02\x01"[..])
            && header.get(18..20) == Some(&(if arm { 183u16 } else { 62u16 }).to_le_bytes()[..])
    } else if target.contains("darwin") {
        header.get(..4) == Some(&[0xcf, 0xfa, 0xed, 0xfe][..])
            && header.get(4..8)
                == Some(&(if arm { 0x100000cu32 } else { 0x1000007u32 }).to_le_bytes()[..])
    } else {
        let offset = header
            .get(60..64)
            .map(|b| u32::from_le_bytes(b.try_into().expect("PE offset")) as usize);
        header.starts_with(b"MZ")
            && offset.is_some_and(|p| {
                // Supported targets are 64-bit; a u32 offset cannot overflow usize.
                header.get(p..p + 4) == Some(&b"PE\0\0"[..])
                    && header.get(p + 4..p + 6)
                        == Some(&(if arm { 0xaa64u16 } else { 0x8664u16 }).to_le_bytes()[..])
            })
    };
    if valid {
        Ok(())
    } else {
        Err(failed(
            "archive binary does not match this operating system/architecture; nothing was installed",
        ))
    }
}
