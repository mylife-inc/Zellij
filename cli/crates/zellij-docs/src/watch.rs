//! Portable preview watching. Poll actual bytes, debounce saves, and publish
//! complete builds serially. No native watcher service or external tool needed.
use crate::{BuildOptions, BuildReport, PreparedBuild};
use camino::{Utf8Path, Utf8PathBuf};
use std::{
    collections::{BTreeMap, hash_map::DefaultHasher},
    fs::{self, File},
    hash::Hasher,
    io::{self, Read},
    sync::{
        Arc, RwLock, RwLockReadGuard,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use zellij_core::{Result, files};

pub struct LiveReload {
    session: String,
    state: RwLock<Revision>,
}
pub(crate) struct Revision {
    generation: u64,
    pub error: Option<String>,
}
impl Default for LiveReload {
    fn default() -> Self {
        Self {
            session: format!(
                "{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            ),
            state: RwLock::new(Revision {
                generation: 0,
                error: None,
            }),
        }
    }
}
impl LiveReload {
    pub(crate) fn read(&self) -> RwLockReadGuard<'_, Revision> {
        self.state.read().unwrap_or_else(|e| e.into_inner())
    }
    pub(crate) fn version(&self, revision: &Revision) -> String {
        format!("{}-{}", self.session, revision.generation)
    }
    pub fn failed(&self, error: String) {
        self.state.write().unwrap_or_else(|e| e.into_inner()).error = Some(error);
    }
    pub fn publish(&self, build: PreparedBuild) -> Result<BuildReport> {
        // HTTP holds the read side only while opening files/creating responses,
        // never while writing to a possibly stalled client. It cannot observe
        // the directory swap gap or an HTML/revision mismatch.
        let mut revision = self.state.write().unwrap_or_else(|e| e.into_inner());
        let report = build.publish()?;
        revision.generation += 1;
        revision.error = None;
        Ok(report)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot(BTreeMap<Utf8PathBuf, (u64, u64)>);
pub(crate) fn is_link(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_type().is_symlink() || metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    metadata.file_type().is_symlink()
}
impl Snapshot {
    pub fn read(root: &Utf8Path) -> Result<Self> {
        let mut files = BTreeMap::new();
        let mut directories = vec![root.to_owned()];
        while let Some(directory) = directories.pop() {
            // The originally resolved root can disappear or be replaced while
            // watching. Never follow a replacement symlink/junction either.
            if is_link(&fs::symlink_metadata(&directory)?) {
                return Err(files::unsafe_path(
                    &directory,
                    "symbolic links and junctions are not allowed inside documentation",
                ));
            }
            for entry in fs::read_dir(&directory)? {
                let entry = entry?;
                let path = Utf8PathBuf::from_path_buf(entry.path())
                    .map_err(|p| io::Error::other(format!("non-UTF-8 path: {}", p.display())))?;
                let name = path.file_name().unwrap_or_default();
                if crate::ignored_name(name) {
                    continue;
                }
                let metadata = fs::symlink_metadata(&path)?;
                if is_link(&metadata) {
                    return Err(files::unsafe_path(
                        &path,
                        "symbolic links and junctions are not allowed inside documentation",
                    ));
                }
                if metadata.is_dir() {
                    directories.push(path);
                } else if metadata.is_file() {
                    // Hash streamed bytes, so same-size edits and restored timestamps
                    // are detected too. Do not retain asset contents in the snapshot.
                    let mut file = File::open(&path)?;
                    let mut hash = DefaultHasher::new();
                    let mut buffer = [0u8; 65536];
                    let mut size = 0;
                    loop {
                        let count = file.read(&mut buffer)?;
                        if count == 0 {
                            break;
                        }
                        hash.write(&buffer[..count]);
                        size += count as u64;
                    }
                    files.insert(path, (size, hash.finish()));
                } else {
                    return Err(files::unsafe_path(
                        &path,
                        "only regular files and directories are allowed inside documentation",
                    ));
                }
            }
        }
        Ok(Self(files))
    }
}

pub struct WatchOptions {
    pub source: Utf8PathBuf,
    pub output: Utf8PathBuf,
    pub name: Option<String>,
    pub theme: Option<String>,
}
pub struct Watcher {
    stop: Arc<AtomicBool>,
    task: Option<JoinHandle<()>>,
}
impl Watcher {
    pub fn spawn(
        options: WatchOptions,
        initial: Snapshot,
        live: Arc<LiveReload>,
        stop: Arc<AtomicBool>,
    ) -> io::Result<Self> {
        let signal = stop.clone();
        let task = thread::Builder::new()
            .name("zel-watch".into())
            .spawn(move || {
                let mut observed = Ok(initial.clone());
                let mut attempted = Ok(initial);
                while !signal.load(Ordering::Acquire) {
                    thread::sleep(Duration::from_millis(300));
                    if signal.load(Ordering::Acquire) {
                        break;
                    }
                    let current = Snapshot::read(&options.source).map_err(|e| e.to_string());
                    if current != observed {
                        observed = current;
                        continue;
                    }
                    if current == attempted {
                        continue;
                    }
                    attempted = current.clone();
                    let result = current.and_then(|_| {
                        let prepared = crate::prepare(
                            &options.source,
                            &options.output,
                            BuildOptions {
                                name: options.name.as_deref(),
                                theme: options.theme.as_deref(),
                            },
                        )
                        .map_err(|e| e.to_string())?;
                        if signal.load(Ordering::Acquire) {
                            return Ok(None);
                        }
                        live.publish(prepared).map(Some).map_err(|e| e.to_string())
                    });
                    match result {
                        Ok(Some(report)) => {
                            for warning in report.warnings {
                                eprintln!("warning: {warning}");
                            }
                            println!("Rebuilt {} HTML pages; reloading preview.", report.pages);
                        }
                        Ok(None) => break,
                        Err(error) => {
                            eprintln!("Rebuild failed; keeping the last successful site.\n{error}");
                            live.failed(error);
                        }
                    }
                }
            })?;
        Ok(Self {
            stop,
            task: Some(task),
        })
    }
}
impl Drop for Watcher {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(task) = self.task.take() {
            let _ = task.join();
        }
    }
}
