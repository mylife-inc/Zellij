//! The engine, run in a container.
//!
//! `check` is the cheap one: the base image carries the engine's CLI as
//! bundled JavaScript at `/opt/zellij`, with nothing installed, so validating
//! a folder is a mount and a `node` invocation. No `npm install`, no Next.js,
//! about a second.
//!
//! `build` and `dev` are not cheap, because they run the real application.

use camino::Utf8Path;
use std::net::UdpSocket;
use zellij_core::{Error, Result};

use crate::runtime::{METADATA_TIMEOUT, Probe, Runtime, probe_output, probe_status};
use crate::{BuildMode, CheckOptions, CheckReport, Engine};

/// Where the engine's own CLI lives inside the image.
const ENGINE_CLI: &str = "/opt/zellij/bin/zellij.mjs";

/// Where a mounted content folder appears inside the image.
///
/// Not `/app/content`: the wrapper lives at `/app`, and mounting over part of
/// it would replace a directory the image put there. A separate path means the
/// mount can never collide with the image's own layout.
const MOUNTED_CONTENT: &str = "/content";

/// Where the wrapper app inside the image looks for content.
///
/// `check` mounts somewhere neutral because it only reads. `dev` and `build`
/// run the application, and the application was generated with this path.
const WRAPPER_CONTENT: &str = "/app/content";

pub struct ContainerEngine {
    runtime: Runtime,
    image: String,
}

enum ExportLayout {
    Flat,
    Deployment,
    Preview,
}

/// A hosted preview is validated and may be discarded before replacing output.
pub struct PreparedPreview {
    output: crate::output::StaticOutput,
    pub base_path: String,
}
impl PreparedPreview {
    pub fn directory(&self) -> &Utf8Path {
        &self.output.directory
    }
    pub fn publish(self) -> Result<()> {
        self.output.publish()
    }
}

fn take_prefix(output: &crate::output::StaticOutput) -> Result<String> {
    output.ensure_nonempty()?;
    let metadata = output.directory.join(".zellij-deployment.json");
    let prefix =
        serde_json::from_slice(&std::fs::read(&metadata)?).map_err(|_| Error::BuildFailed)?;
    std::fs::remove_file(metadata)?;
    Ok(prefix)
}

/// Rust canonicalizes Windows paths with a verbatim prefix that Docker's bind
/// parser rejects. Preserve the drive/UNC path, spaces and Unicode for clients.
fn bind_path(path: &Utf8Path) -> String {
    let path = path.as_str();
    if let Some(unc) = path.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{unc}")
    } else {
        path.strip_prefix(r"\\?\").unwrap_or(path).into()
    }
}

impl ContainerEngine {
    pub fn new(runtime: Runtime, image: impl Into<String>) -> Self {
        Self {
            runtime,
            image: image.into(),
        }
    }

    /// The default image, unless `ZELLIJ_IMAGE` says otherwise — which is what
    /// an organisation mirroring it internally needs.
    pub fn detect(image: Option<&str>) -> Result<Self> {
        let runtime = crate::runtime::detect()?;
        let image = crate::configured_image(image);

        Ok(Self::new(runtime, image))
    }

    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }

    pub fn image(&self) -> &str {
        &self.image
    }

    /// Explicitly download/update the selected engine; never start a daemon.
    pub fn pull(&self) -> Result<()> {
        let status = self
            .runtime
            .command()
            .args(["pull", &self.image])
            .status()?;
        if status.success() {
            Ok(())
        } else {
            Err(Error::ImageOperationFailed { operation: "pull" })
        }
    }

    /// Is the image already on this machine? None means inspection was
    /// unavailable or timed out; it must not be reported as a missing image.
    pub fn image_is_local(&self) -> Option<bool> {
        match probe_status(
            self.runtime
                .command()
                .args(["image", "inspect", &self.image]),
            METADATA_TIMEOUT,
        ) {
            Probe::Ready => Some(true),
            Probe::Failed => Some(false),
            _ => None,
        }
    }

    /// The mount argument for a content folder, read-only.
    ///
    /// Read-only because the engine has no business writing into somebody's
    /// content, and a container that cannot is a container that will not.
    fn content_mount(content: &Utf8Path) -> Result<String> {
        Self::mount_at(content, MOUNTED_CONTENT)
    }

    fn mount_at(content: &Utf8Path, target: &str) -> Result<String> {
        let absolute = content
            .canonicalize_utf8()
            .map_err(|_| Error::NoContentFolder {
                path: content.to_owned(),
            })?;

        Ok(format!("{}:{target}:ro", bind_path(&absolute)))
    }

    /// A named volume for the wrapper's `node_modules`, keyed by image.
    ///
    /// The base image installs nothing, so the first `zel dev` spends a couple
    /// of minutes on `npm install`; keeping the result means the second starts
    /// in seconds. Keyed by image because the four feature variants ask for
    /// different dependencies, and sharing one volume between them would
    /// install maths into the image that does not want it.
    fn modules_volume(&self) -> String {
        let key: String = self
            .image
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
            .collect();

        // Keyed by image *identity*, not by its tag.
        //
        // `:latest` is a moving target. Keying on the name alone means one
        // volume serves every version the tag ever pointed at, and
        // `npm install` reuses what it finds — so a freshly pulled image with a
        // new engine is shadowed by whatever was installed months ago.
        //
        // That is not a stale cache in the harmless sense. It produced:
        //
        //   examples/matrix.yaml: unknown section type "matrix".
        //
        // from an image that knew the type perfectly well, and sent two people
        // looking at the registry. The volume held an engine that knew `zigzag`
        // and not `hero-split`, and no amount of rebuilding the image would
        // have fixed it.
        //
        // The id changes whenever the image does, so a new image gets a new
        // volume and the old one simply stops being used. Falling back to the
        // tag when the id cannot be read keeps this working before the first
        // pull, which is the one case where there is nothing stale to shadow.
        match self.image_id() {
            Some(id) => format!("zellij-modules-{key}-{}", &id[..id.len().min(12)]),
            None => format!("zellij-modules-{key}"),
        }
    }

    /// The image's content id, if it is on this machine.
    fn image_id(&self) -> Option<String> {
        let id = probe_output(
            self.runtime
                .command()
                .args(["image", "inspect", "--format", "{{.Id}}", &self.image]),
            METADATA_TIMEOUT,
        )?;
        let id = id.trim().rsplit(':').next()?.to_string();
        (!id.is_empty()).then_some(id)
    }

    /// A development server, with the content folder mounted live.
    ///
    /// Runs in the foreground and streams the engine's output, so Ctrl-C stops
    /// it the way Ctrl-C stops anything. `--init` gives the container a real
    /// PID 1, without which the signal reaches a shell that ignores it.
    pub fn dev(&self, content: &Utf8Path, port: u16) -> Result<()> {
        let mount = Self::mount_at(content, WRAPPER_CONTENT)?;
        let ports = format!("{port}:3000");
        let volume = format!("{}:/app/node_modules", self.modules_volume());

        // Validate, install, serve — one container, in that order. Checking
        // first means a broken folder is reported in a second rather than
        // after a two-minute install.
        let script = guarded(
            WRAPPER_CONTENT,
            &format!(
                "node {ENGINE_CLI} check {WRAPPER_CONTENT} && \
                 npm install --install-links --no-audit --no-fund --loglevel=error && \
                 npm run dev -- --hostname 0.0.0.0"
            ),
        );

        let origins = format!("ZELLIJ_DEV_ORIGINS={}", dev_origins());

        let status = self
            .runtime
            .command()
            .args(["run", "--rm", "--init"])
            .args(["-v", &mount])
            .args(["-v", &volume])
            .args(["-p", &ports])
            .args(["-e", &origins])
            .arg(&self.image)
            .args(["sh", "-c", &script])
            .status()?;

        match status.code() {
            Some(EMPTY_MOUNT) => Err(Error::MountEmpty {
                path: content.to_owned(),
            }),
            // 130 is Ctrl-C, which is how this command is meant to end.
            Some(0) | Some(130) | None => Ok(()),
            Some(_) => Err(Error::ContentInvalid),
        }
    }
}

/// Origins the dev server inside the container should answer to.
///
/// Next's dev server refuses cross-origin requests for `/_next/*`, and trusts
/// only `localhost` and the address it binds. Inside a container that address is
/// the container's, so opening the site from a phone on the same network is
/// refused: the client chunks never load, nothing hydrates, and the page renders
/// as bare chrome above an empty body — which looks like a broken site rather
/// than a blocked request.
///
/// The engine's Next config reads this variable and adds it to
/// `allowedDevOrigins`. Only the host's own address goes in, so it permits the
/// machine you are working on and nothing else. Anything already in the
/// environment is preserved, which is how a tunnel — ngrok, Tailscale, a
/// forwarded Codespaces port — gets added.
fn dev_origins() -> String {
    let existing = std::env::var("ZELLIJ_DEV_ORIGINS").unwrap_or_default();

    match lan_address() {
        Some(address) if existing.is_empty() => address,
        Some(address) => format!("{existing},{address}"),
        None => existing,
    }
}

/// The address this machine is reachable at from the rest of the network.
///
/// Asked of the routing table rather than of the interface list: a laptop has
/// several addresses — loopback, a bridge per container runtime, a VPN — and the
/// one a phone can reach is whichever the default route would use.
///
/// Nothing is sent. Connecting a UDP socket only selects a route, which is
/// enough for the kernel to say which local address it would send from. The
/// destination is TEST-NET-1, reserved for documentation and routed nowhere, so
/// there is no host on the other end even in principle.
fn lan_address() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("192.0.2.1:80").ok()?;
    Some(socket.local_addr().ok()?.ip().to_string())
}

/// The `chown` that returns build output to the invoking user, or nothing.
///
/// Unix only, and deliberately best-effort: `|| true` so a runtime that maps
/// ownership for us — Docker Desktop and Colima both virtualise it on macOS —
/// cannot turn a successful build into a failure over a no-op.
#[cfg(unix)]
fn own_output_as(out: &Utf8Path) -> String {
    use std::os::unix::fs::MetadataExt;

    match std::fs::metadata(out) {
        // Braces, so `|| true` applies to the chown and NOT to the whole `&&`
        // chain before it. Without them a failed build was swallowed and
        // `zel build` printed a tick over an empty output directory — the
        // build error scrolled past above a success line.
        Ok(meta) => format!(
            " && {{ chown -R {}:{} /out || true; }}",
            meta.uid(),
            meta.gid()
        ),
        // If the directory cannot be read, the build is about to fail on the
        // mount anyway, and with a better message than this could give.
        Err(_) => String::new(),
    }
}

/// Windows has no uid to hand back to.
#[cfg(not(unix))]
fn own_output_as(_out: &Utf8Path) -> String {
    String::new()
}

/// Exit code the wrapper uses to say the mount arrived empty.
///
/// A number no engine command returns, so it cannot be confused with a real
/// failure. 97 is arbitrary and outside anything Node or the checker uses.
const EMPTY_MOUNT: i32 = 97;

/// Exit code the wrapper uses to say the content itself failed validation.
///
/// The build script runs `check` before it runs anything else, so once that has
/// passed, a later failure is provably not the content's fault. Without a code
/// of its own, every failure after it — a dependency conflict, a network blip,
/// an out-of-memory kill — was reported as "the content folder is not valid",
/// which sent us reading YAML while npm was the thing that had failed.
const CONTENT_INVALID: i32 = 96;

/// Check the mount before trusting it.
///
/// A bind mount of a path the runtime does not share does not fail — it
/// succeeds, and the directory inside is empty. The checker then reports a
/// content root with no pages, which is true of what it was shown and a lie
/// about what was asked. Colima shares `$HOME` and not `/tmp`, so a folder in
/// the wrong place produces a confident, wrong answer.
///
/// Folded into the same invocation rather than probed separately: a second
/// container start would double the latency of the fast command.
fn guarded(mount: &str, command: &str) -> String {
    format!("if [ -z \"$(ls -A {mount} 2>/dev/null)\" ]; then exit {EMPTY_MOUNT}; fi; {command}")
}

impl Engine for ContainerEngine {
    fn check(&self, content: &Utf8Path, options: CheckOptions) -> Result<CheckReport> {
        let mount = Self::content_mount(content)?;

        let output = self
            .runtime
            .command()
            .args(["run", "--rm", "-v", &mount, &self.image])
            .args([
                "sh",
                "-c",
                &guarded(
                    MOUNTED_CONTENT,
                    &format!(
                        "exec node {ENGINE_CLI} check {MOUNTED_CONTENT}{}{}",
                        if options.deep { " --deep" } else { "" },
                        if options.strict { " --strict" } else { "" }
                    ),
                ),
            ])
            .output()?;

        if output.status.code() == Some(EMPTY_MOUNT) {
            return Err(Error::MountEmpty {
                path: content.to_owned(),
            });
        }

        // The engine prints its findings to both streams, and a reader wants
        // them in one piece rather than sorted by which one they came from.
        let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
        text.push_str(&String::from_utf8_lossy(&output.stderr));

        // The mounted path is an implementation detail. Somebody who ran
        // `zel check ./docs` should read about ./docs.
        let text = text.replace(MOUNTED_CONTENT, content.as_str());

        Ok(CheckReport {
            ok: output.status.success(),
            output: text,
        })
    }

    /// Build the site into `out`.
    ///
    /// Only `Static` so far. The container writes the export to a mounted
    /// directory rather than to a volume we then copy out of: one fewer moving
    /// part, and the result is on the host the moment the build finishes.
    fn build(&self, content: &Utf8Path, out: &Utf8Path, mode: BuildMode) -> Result<()> {
        if mode != BuildMode::Static {
            return Err(Error::NoEngine {
                reason: "only --target static is implemented so far".into(),
            });
        }
        self.prepare_export(content, out, ExportLayout::Flat)?
            .publish()
    }
}

impl ContainerEngine {
    /// Export once into an HTTP-root layout, mounting files at the same basePath
    /// resolved by Next. Ordinary `build` exports retain their existing layout.
    pub fn build_for_deployment(&self, content: &Utf8Path, out: &Utf8Path) -> Result<String> {
        let output = self.prepare_export(content, out, ExportLayout::Deployment)?;
        let prefix = take_prefix(&output)?;
        output.publish()?;
        Ok(prefix)
    }

    /// Flat export files, with the actual prefix resolved by the engine. The
    /// caller validates that prefix and controls publication/cancellation.
    pub fn prepare_preview(&self, content: &Utf8Path, out: &Utf8Path) -> Result<PreparedPreview> {
        let output = self.prepare_export(content, out, ExportLayout::Preview)?;
        let base_path = take_prefix(&output)?;
        Ok(PreparedPreview { output, base_path })
    }

    fn prepare_export(
        &self,
        content: &Utf8Path,
        out: &Utf8Path,
        layout: ExportLayout,
    ) -> Result<crate::output::StaticOutput> {
        let output = crate::output::StaticOutput::prepare(content, out)?;
        let mount = Self::mount_at(content, WRAPPER_CONTENT)?;
        let volume = format!("{}:/app/node_modules", self.modules_volume());
        let out_mount = format!("{}:/out", bind_path(&output.directory));

        // Give the output back to whoever asked for it.
        //
        // The build runs as root, because it must: `npm install` writes into
        // the image's own `/app`, and a container told `--user 1000` cannot.
        // But that means everything it copies into the mounted `/out` is owned
        // by root, on a directory the user created — so `rm -rf dist` needs
        // sudo, and any tool that tries to move it fails:
        //
        //   mv: cannot move 'dist' to '/home/runner/work/_temp/site': Permission denied
        //
        // That was a CI failure, but it is not a CI problem: it happens to
        // anyone who runs `zel build` and then tries to delete the folder.
        //
        // The ownership is read from the output directory itself, which was
        // just created by this process and therefore carries the invoking
        // user's ids — no `getuid` binding, and correct under `sudo` too, where
        // the directory belongs to whoever the command really ran as.
        let handback = own_output_as(&output.directory);

        let copy = match layout {
            ExportLayout::Deployment => {
                r#"node -e 'const fs = require("node:fs"); const p = require("./node_modules/@shebka/zellij/deployment-config.cjs").resolveContentLocation().basePath || ""; fs.mkdirSync("/out" + p, {recursive:true}); fs.cpSync("/app/out", "/out" + p, {recursive:true}); fs.writeFileSync("/out/.zellij-deployment.json", JSON.stringify(p));'"#
            }
            ExportLayout::Preview => {
                r#"node -e 'const fs = require("node:fs"); const p = require("./node_modules/@shebka/zellij/deployment-config.cjs").resolveContentLocation().basePath || ""; fs.cpSync("/app/out", "/out", {recursive:true}); fs.writeFileSync("/out/.zellij-deployment.json", JSON.stringify(p));'"#
            }
            ExportLayout::Flat => "cp -a out/. /out/",
        };
        let script = guarded(
            WRAPPER_CONTENT,
            &format!(
                "node {ENGINE_CLI} check {WRAPPER_CONTENT} || exit {CONTENT_INVALID}; \
                 npm install --install-links --no-audit --no-fund --loglevel=error && \
                 npm run build && \
                 {copy}{handback}"
            ),
        );

        let status = self
            .runtime
            .command()
            .args(["run", "--rm", "--init"])
            .args(["-v", &mount])
            .args(["-v", &volume])
            .args(["-v", &out_mount])
            // The image defaults to `standalone`, which is what a site image
            // wants. A folder for a CDN is a different artefact.
            .args(["-e", "ZELLIJ_OUTPUT=export"])
            .arg(&self.image)
            .args(["sh", "-c", &script])
            .status()?;

        match status.code() {
            Some(EMPTY_MOUNT) => Err(Error::MountEmpty {
                path: content.to_owned(),
            }),
            Some(CONTENT_INVALID) => Err(Error::ContentInvalid),
            Some(0) => Ok(output),
            _ => Err(Error::BuildFailed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> ContainerEngine {
        ContainerEngine::new(
            Runtime {
                program: "docker".into(),
                host: None,
                version: "27.0.0".into(),
            },
            "ghcr.io/example/zellij-base:latest",
        )
    }

    #[test]
    fn a_missing_content_folder_is_named_rather_than_mounted() {
        let err =
            ContainerEngine::content_mount(Utf8Path::new("./definitely-not-here")).unwrap_err();
        assert!(matches!(err, Error::NoContentFolder { .. }), "{err:?}");
    }

    #[test]
    fn the_mount_is_absolute_and_read_only() {
        let dir = tempfile::tempdir().unwrap();
        let path = camino::Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();

        let mount = ContainerEngine::content_mount(&path).unwrap();

        let source = mount.strip_suffix(":/content:ro").expect("read-only mount");
        assert!(
            Utf8Path::new(source).is_absolute(),
            "the mount source must be absolute: {mount}"
        );
        assert_eq!(
            Utf8Path::new(source).canonicalize_utf8().unwrap(),
            path.canonicalize_utf8().unwrap()
        );
        assert!(!source.starts_with(r"\\?\"));
    }

    #[test]
    fn bind_sources_use_client_compatible_windows_paths_without_rewriting_unix_names() {
        for (input, expected) in [
            (r"\\?\C:\Docs 日本\content", r"C:\Docs 日本\content"),
            (r"\\?\UNC\server\share\site", r"\\server\share\site"),
            (r"C:\Docs\site", r"C:\Docs\site"),
            (r"/tmp/Docs 日本/\\?\name", r"/tmp/Docs 日本/\\?\name"),
        ] {
            assert_eq!(bind_path(Utf8Path::new(input)), expected);
        }
    }

    #[test]
    fn the_image_can_be_overridden_for_an_internal_mirror() {
        assert_eq!(engine().image(), "ghcr.io/example/zellij-base:latest");
    }
}
