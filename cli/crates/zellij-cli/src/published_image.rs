//! Bounded inspection of the immutable registry receipt after an image push.
use std::{
    collections::BTreeSet,
    io::Read,
    process::Stdio,
    time::{Duration, Instant},
};
use zellij_core::{Error, Result};
use zellij_engine::ContainerEngine;
fn failed(message: impl Into<String>) -> Error {
    Error::DeploymentFailed(message.into())
}
pub(crate) fn validate_digest(repository: &str, image: &str) -> Result<()> {
    let prefix = format!("{repository}@sha256:");
    if !image.strip_prefix(&prefix).is_some_and(|s| {
        s.len() == 64
            && s.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    }) {
        return Err(failed(
            "expected the pushed image's SHA-256 repository digest",
        ));
    }
    Ok(())
}
pub(crate) fn inspect(engine: &ContainerEngine, tag: &str, repository: &str) -> Result<String> {
    // File-backed capture avoids blocked pipes and bounds output before parsing.
    let output = tempfile::NamedTempFile::new()?;
    let mut command = engine.runtime().command();
    command
        .args(["image", "inspect", "--format", "{{json .RepoDigests}}", tag])
        .stdin(Stdio::null())
        .stdout(output.reopen()?)
        .stderr(Stdio::null());
    struct Child(std::process::Child);
    impl Drop for Child {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let mut child = Child(
        command
            .spawn()
            .map_err(|_| failed("cannot inspect the pushed image"))?,
    );
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if output.as_file().metadata()?.len() > 64 * 1024 {
            return Err(failed("image inspection output exceeded its limit"));
        }
        if let Some(status) = child.0.try_wait()? {
            if !status.success() {
                return Err(failed(
                    "cannot inspect the pushed image; no deployment started",
                ));
            }
            break;
        }
        if Instant::now() >= deadline {
            return Err(failed("image inspection timed out; no deployment started"));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let mut bytes = Vec::new();
    output
        .reopen()?
        .take(64 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 64 * 1024 {
        return Err(failed("image inspection output exceeded its limit"));
    }
    let values: Vec<String> = serde_json::from_slice(&bytes).map_err(|_| {
        failed(
            "container runtime did not return repository digests after push; no deployment started",
        )
    })?;
    select_digest(repository, values)
}
fn select_digest(repository: &str, values: Vec<String>) -> Result<String> {
    let digests: BTreeSet<_> = values
        .into_iter()
        .filter_map(|value| {
            let (name, digest) = value.split_once('@')?;
            let docker_hub = repository.strip_prefix("docker.io/");
            let matches = name == repository
                || docker_hub.is_some_and(|short| {
                    name == short
                        || name.strip_prefix("index.docker.io/") == Some(short)
                        || short.strip_prefix("library/") == Some(name)
                });
            let canonical = format!("{repository}@{digest}");
            (matches && validate_digest(repository, &canonical).is_ok()).then_some(canonical)
        })
        .collect();
    if digests.len() != 1 {
        return Err(failed(
            "container runtime did not return one unambiguous repository digest; no deployment started",
        ));
    }
    Ok(digests.into_iter().next().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn docker_hub_receipts_are_normalized_and_ambiguous_digests_rejected() {
        let hash = format!("sha256:{}", "a".repeat(64));
        for alias in [
            "acme/docs",
            "docker.io/acme/docs",
            "index.docker.io/acme/docs",
        ] {
            assert_eq!(
                select_digest("docker.io/acme/docs", vec![format!("{alias}@{hash}")]).unwrap(),
                format!("docker.io/acme/docs@{hash}")
            );
        }
        assert!(select_digest("ghcr.io/acme/docs", vec![format!("acme/docs@{hash}")]).is_err());
        assert!(
            select_digest(
                "ghcr.io/acme/docs",
                vec![
                    format!("ghcr.io/acme/docs@{hash}"),
                    format!("ghcr.io/acme/docs@sha256:{}", "b".repeat(64))
                ]
            )
            .is_err()
        );
        assert!(
            select_digest(
                "ghcr.io/acme/docs",
                vec!["ghcr.io/acme/docs@sha256:bad".into()]
            )
            .is_err()
        );
    }
}
