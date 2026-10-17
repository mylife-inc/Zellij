//! Loopback preview of an existing export. Files are served unchanged; this is
//! not a build watcher, deployment server, or SPA fallback.
use camino::{Utf8Path, Utf8PathBuf};
use std::{
    fs::File,
    io,
    net::TcpListener,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
};
use tiny_http::{Header, Method, Request, Response, ResponseBox};
use zellij_core::{Error, Result, files};

use crate::links::{decode, encode};

pub fn listener(port: u16) -> Result<TcpListener> {
    TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port)).map_err(|error| {
        if error.kind() == io::ErrorKind::AddrInUse {
            Error::InvalidArgument(format!(
                "port {port} is already in use; choose another --port (or --port 0)"
            ))
        } else {
            error.into()
        }
    })
}

pub struct StaticServer {
    http: tiny_http::Server,
    site: Arc<Site>,
    url: String,
}

struct Site {
    root: Utf8PathBuf,
    prefix: String,
    port: u16,
}

enum Resource {
    File(File, Utf8PathBuf),
    Redirect(String),
}

fn invalid(message: &str) -> Error {
    Error::InvalidArgument(message.into())
}

/// Decode once, then reject traversal, platform aliases and metadata paths.
fn url_path(raw: &str) -> std::result::Result<String, u16> {
    let path = decode(raw).ok_or(400u16)?;
    if !path.starts_with('/')
        || path.contains("//")
        || path.contains(['\\', ':', '?', '#'])
        || path.chars().any(char::is_control)
    {
        return Err(400);
    }
    let relative = path.strip_prefix('/').unwrap().trim_end_matches('/');
    if relative.is_empty() {
        return if path == "/" { Ok(path) } else { Err(400) };
    }
    crate::portable_path(Utf8Path::new(relative)).map_err(|_| 400u16)?;
    if relative
        .split('/')
        .any(|part| part.starts_with('.') && part != ".well-known")
    {
        return Err(403);
    }
    Ok(path)
}

impl StaticServer {
    pub fn bind(root: &Utf8Path, port: u16, base_path: &str) -> Result<Self> {
        Self::from_listener(root, listener(port)?, base_path)
    }

    /// `view` reserves the port before building, then supplies that same socket.
    pub fn from_listener(root: &Utf8Path, listener: TcpListener, base_path: &str) -> Result<Self> {
        let root = root.canonicalize_utf8()?;
        if !root.is_dir() {
            return Err(invalid("serve expects a generated site directory"));
        }
        let prefix = url_path(base_path).map_err(|_| {
            invalid("--base-path must be an absolute URL path without traversal, query or fragment")
        })?;
        let prefix = prefix.trim_end_matches('/').to_owned();
        let port = listener.local_addr()?.port();
        let site = Site { root, prefix, port };
        match site.file("index.html") {
            Ok(Some(path)) if path.is_file() => {}
            _ => {
                return Err(invalid(
                    "serve requires a regular index.html inside the site directory; run zel build first",
                ));
            }
        }
        let url = format!("http://127.0.0.1:{port}{}/", encode(&site.prefix));
        let http = tiny_http::Server::from_listener(listener, None).map_err(io::Error::other)?;
        Ok(Self {
            http,
            site: Arc::new(site),
            url,
        })
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn run(self, stop: &AtomicBool) -> Result<()> {
        // Bound concurrent response writers. The control thread never waits for
        // a slow reader, so Ctrl-C can stop the CLI even during a large download.
        // Dropping this server releases its listener; process exit closes any
        // in-flight response sockets, without modifying the served files.
        let active = Arc::new(AtomicUsize::new(0));
        while !stop.load(Ordering::Acquire) {
            if active.load(Ordering::Acquire) >= 16 {
                thread::sleep(Duration::from_millis(10));
                continue;
            }
            let Some(request) = self.http.recv_timeout(Duration::from_millis(100))? else {
                continue;
            };
            let site = self.site.clone();
            active.fetch_add(1, Ordering::AcqRel);
            let permit = ResponsePermit(active.clone());
            thread::Builder::new()
                .name("zel-http-response".into())
                .spawn(move || {
                    let _permit = permit;
                    let response = site.response(&request);
                    if let Err(error) = request.respond(response) {
                        eprintln!("warning: could not finish HTTP response: {error}");
                    }
                })?;
        }
        Ok(())
    }
}

struct ResponsePermit(Arc<AtomicUsize>);
impl Drop for ResponsePermit {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

impl Site {
    fn file(&self, relative: &str) -> std::result::Result<Option<Utf8PathBuf>, u16> {
        let path =
            files::contained_path(&self.root, Utf8Path::new(relative)).map_err(|_| 403u16)?;
        // Reject Windows junctions and other reparse points as well as symlinks.
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            let mut component = self.root.clone();
            for part in relative.split('/') {
                component.push(part);
                if std::fs::symlink_metadata(&component)
                    .is_ok_and(|m| m.file_attributes() & 0x400 != 0)
                {
                    return Err(403);
                }
            }
        }
        match path.canonicalize_utf8() {
            Ok(canonical) if canonical.starts_with(&self.root) => Ok(Some(canonical)),
            Ok(_) => Err(403),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(_) => Err(403),
        }
    }

    fn resolve(&self, target: &str) -> std::result::Result<Resource, u16> {
        let (raw, query) = target.split_once('?').map_or((target, ""), |(p, q)| (p, q));
        let path = url_path(raw)?;
        let local = if self.prefix.is_empty() {
            path.as_str()
        } else if path == self.prefix {
            ""
        } else {
            path.strip_prefix(&self.prefix)
                .filter(|p| p.starts_with('/'))
                .ok_or(404u16)?
        };
        let relative = local.trim_matches('/');
        let file = if relative.is_empty() {
            Some(self.root.clone())
        } else {
            self.file(relative)?
        };
        if let Some(file) = file {
            if file.is_dir() {
                let index = if relative.is_empty() {
                    "index.html".into()
                } else {
                    format!("{relative}/index.html")
                };
                let index = self.file(&index)?.filter(|p| p.is_file()).ok_or(404u16)?;
                if !path.ends_with('/') {
                    let query = if query.is_empty() {
                        String::new()
                    } else {
                        format!("?{query}")
                    };
                    return Ok(Resource::Redirect(format!("{}/{query}", encode(&path))));
                }
                return Self::open(index);
            }
            if file.is_file() && !local.ends_with('/') {
                return Self::open(file);
            }
            return Err(404);
        }
        if !local.ends_with('/') {
            if let Some(file) = self
                .file(&format!("{relative}.html"))?
                .filter(|p| p.is_file())
            {
                return Self::open(file);
            }
        }
        Err(404)
    }

    fn open(path: Utf8PathBuf) -> std::result::Result<Resource, u16> {
        File::open(&path)
            .map(|file| Resource::File(file, path))
            .map_err(|error| {
                if error.kind() == io::ErrorKind::NotFound {
                    404
                } else {
                    500
                }
            })
    }

    fn response(&self, request: &Request) -> ResponseBox {
        let mut response = if !matches!(request.method(), Method::Get | Method::Head) {
            text_response(405, "Method not allowed\n").with_header(header("Allow", "GET, HEAD"))
        } else if request.headers().iter().any(|h| {
            let host = h.value.as_str().to_ascii_lowercase();
            h.field.equiv("Host")
                && !(self.port == 80 && ["localhost", "127.0.0.1"].contains(&host.as_str()))
                && ![
                    format!("127.0.0.1:{}", self.port),
                    format!("localhost:{}", self.port),
                ]
                .contains(&host)
        }) {
            text_response(403, "Forbidden host\n")
        } else {
            match self.resolve(request.url()) {
                Ok(Resource::File(file, path)) => file_response(file, &path, 200),
                Ok(Resource::Redirect(location)) => {
                    match Header::from_bytes("Location", location) {
                        Ok(location) => Response::empty(308).with_header(location).boxed(),
                        Err(_) => text_response(400, "Bad request\n"),
                    }
                }
                Err(404) => match self.file("404.html").ok().flatten().filter(|p| p.is_file()) {
                    Some(path) => match File::open(&path) {
                        Ok(file) => file_response(file, &path, 404),
                        Err(_) => text_response(404, "Not found\n"),
                    },
                    None => text_response(404, "Not found\n"),
                },
                Err(400) => text_response(400, "Bad request\n"),
                Err(403) => text_response(403, "Forbidden path\n"),
                Err(_) => text_response(500, "Could not read file\n"),
            }
        };
        response.add_header(header("Cache-Control", "no-store"));
        response.add_header(header("X-Content-Type-Options", "nosniff"));
        response
    }
}

fn header(name: &str, value: &str) -> Header {
    Header::from_bytes(name, value).expect("static valid HTTP header")
}
fn text_response(status: u16, text: &str) -> ResponseBox {
    Response::from_string(text).with_status_code(status).boxed()
}
fn file_response(file: File, path: &Utf8Path, status: u16) -> ResponseBox {
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    let charset = mime.type_() == "text"
        || [
            "application/javascript",
            "application/json",
            "image/svg+xml",
        ]
        .contains(&mime.essence_str());
    let content_type = if charset {
        format!("{mime}; charset=utf-8")
    } else {
        mime.to_string()
    };
    Response::from_file(file)
        .with_status_code(status)
        .with_header(header("Content-Type", &content_type))
        .boxed()
}
