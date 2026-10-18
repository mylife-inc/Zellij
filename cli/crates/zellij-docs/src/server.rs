//! Loopback preview. Ordinary serving leaves bytes unchanged; an explicitly
//! enabled watch preview injects its reload client only into HTTP responses.
use camino::{Utf8Path, Utf8PathBuf};
use futures_util::TryStreamExt;
use http_body_util::{BodyExt, Full, StreamBody, combinators::UnsyncBoxBody};
use hyper::{
    Method, Request, Response,
    body::{Bytes, Frame},
    header::HeaderValue,
    server::conn::http1,
    service::service_fn,
};
use hyper_util::rt::{TokioIo, TokioTimer};
use std::{
    convert::Infallible,
    fs::File,
    io::{self, Read},
    net::TcpListener,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio_util::io::ReaderStream;

type Body = UnsyncBoxBody<Bytes, io::Error>;
type HttpResponse = Response<Body>;
use zellij_core::{Error, Result, files};

use crate::links::{decode, encode};
use crate::watch::LiveReload;

const WATCH_NOT_FOUND: &[u8] = br#"<!doctype html>
<html lang="en"><head><meta charset="utf-8"><title>Page not found</title></head>
<body><h1>Page not found</h1><p>This page is absent from the latest successful
build. Restore it or navigate back to another page.</p></body></html>"#;

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
    listener: TcpListener,
    site: Arc<Site>,
    url: String,
}

struct Site {
    root: Utf8PathBuf,
    prefix: String,
    port: u16,
    live: Option<Arc<LiveReload>>,
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
        let site = Site {
            root,
            prefix,
            port,
            live: None,
        };
        match site.file("index.html") {
            Ok(Some(path)) if path.is_file() => {}
            _ => {
                return Err(invalid(
                    "serve requires a regular index.html inside the site directory; run zel build first",
                ));
            }
        }
        let url = format!("http://127.0.0.1:{port}{}/", encode(&site.prefix));
        Ok(Self {
            listener,
            site: Arc::new(site),
            url,
        })
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn with_reload(mut self, live: Arc<LiveReload>) -> Self {
        // The site is only cloned by run(), which consumes this server.
        Arc::get_mut(&mut self.site)
            .expect("server has not started")
            .live = Some(live);
        self
    }

    pub fn run(self, stop: &AtomicBool) -> Result<()> {
        // Idle keep-alive sockets are async tasks, never blocked worker threads.
        // File lookup/read work uses a bounded blocking pool; file bodies stream
        // with backpressure instead of loading whole downloads into memory.
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .max_blocking_threads(16)
            .build()?;
        let result = runtime.block_on(self.serve(stop));
        // serve drops its JoinSet, cancelling all connection tasks. Shutdown
        // does not wait for a slow network reader or an idle persistent socket.
        runtime.shutdown_timeout(Duration::from_secs(1));
        result
    }

    async fn serve(self, stop: &AtomicBool) -> Result<()> {
        self.listener.set_nonblocking(true)?;
        let listener = tokio::net::TcpListener::from_std(self.listener)?;
        let mut connections = tokio::task::JoinSet::new();
        let mut shutdown = tokio::time::interval(Duration::from_millis(100));
        while !stop.load(Ordering::Acquire) {
            tokio::select! {
                _ = shutdown.tick() => {}
                _ = connections.join_next(), if !connections.is_empty() => {}
                accepted = listener.accept(), if connections.len() < 256 => {
                    let (stream, _) = match accepted {
                        Ok(connection) => connection,
                        Err(error) if matches!(error.kind(), io::ErrorKind::Interrupted | io::ErrorKind::ConnectionAborted) => continue,
                        Err(error) => return Err(error.into()),
                    };
                    let site = self.site.clone();
                    connections.spawn(async move {
                        let service = service_fn(move |request: Request<hyper::body::Incoming>| {
                            let site = site.clone();
                            async move {
                                // No request body is needed by this GET/HEAD server.
                                let (parts, _) = request.into_parts();
                                let response = tokio::task::spawn_blocking(move || {
                                    site.response(&Request::from_parts(parts, ()))
                                }).await.unwrap_or_else(|_| text_response(500, "Could not read file\n"));
                                Ok::<_, Infallible>(response)
                            }
                        });
                        // Malformed/disconnected clients end only their own connection.
                        let _ = http1::Builder::new()
                            .timer(TokioTimer::new())
                            .header_read_timeout(Duration::from_secs(10))
                            .max_buf_size(32 * 1024)
                            .serve_connection(TokioIo::new(stream), service)
                            .await;
                    });
                }
            }
        }
        Ok(())
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
        if !local.ends_with('/')
            && let Some(file) = self
                .file(&format!("{relative}.html"))?
                .filter(|p| p.is_file())
        {
            return Self::open(file);
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

    fn response(&self, request: &Request<()>) -> HttpResponse {
        let revision = self.live.as_ref().map(|live| live.read());
        let version = self
            .live
            .as_ref()
            .zip(revision.as_ref())
            .map(|(live, state)| live.version(state));
        let reload = version
            .as_deref()
            .map(|v| (v, revision.as_ref().and_then(|s| s.error.as_deref())));
        let mut response = if !matches!(*request.method(), Method::GET | Method::HEAD) {
            let mut response = text_response(405, "Method not allowed\n");
            response
                .headers_mut()
                .insert("allow", HeaderValue::from_static("GET, HEAD"));
            response
        } else if request.headers().get_all("host").iter().any(|value| {
            let host = value.to_str().unwrap_or_default().to_ascii_lowercase();
            !(self.port == 80 && ["localhost", "127.0.0.1"].contains(&host.as_str()))
                && ![
                    format!("127.0.0.1:{}", self.port),
                    format!("localhost:{}", self.port),
                ]
                .contains(&host)
        }) {
            text_response(403, "Forbidden host\n")
        } else {
            self.content_response(
                request.uri().path_and_query().map_or("/", |p| p.as_str()),
                reload,
            )
        };
        response
            .headers_mut()
            .insert("cache-control", HeaderValue::from_static("no-store"));
        response.headers_mut().insert(
            "x-content-type-options",
            HeaderValue::from_static("nosniff"),
        );
        if request.method() == Method::HEAD {
            *response.body_mut() = full(Bytes::new());
        }
        response
    }

    fn content_response(&self, target: &str, reload: Option<(&str, Option<&str>)>) -> HttpResponse {
        if let Some((version, error)) = reload {
            let path = url_path(target.split('?').next().unwrap_or_default());
            if path.as_deref() == Ok(&format!("{}/_zellij/preview.json", self.prefix)) {
                return bytes_response(
                    200,
                    serde_json::json!({"version": version, "error": error}).to_string(),
                    "application/json; charset=utf-8",
                );
            }
            if path.as_deref() == Ok(&format!("{}/_zellij/preview.js", self.prefix)) {
                return bytes_response(
                    200,
                    include_str!("../assets/preview.js"),
                    "text/javascript; charset=utf-8",
                );
            }
        }
        let version = reload.map(|r| r.0);
        match self.resolve(target) {
            Ok(Resource::File(file, path)) => self.file_response(file, &path, 200, version),
            Ok(Resource::Redirect(location)) => match HeaderValue::from_str(&location) {
                Ok(location) => {
                    let mut response = text_response(308, "");
                    response.headers_mut().insert("location", location);
                    response
                }
                Err(_) => text_response(400, "Bad request\n"),
            },
            Err(404) => match self.file("404.html").ok().flatten().filter(|p| p.is_file()) {
                Some(path) => match File::open(&path) {
                    Ok(file) => self.file_response(file, &path, 404, version),
                    Err(_) => text_response(404, "Not found\n"),
                },
                None => match version {
                    Some(version) => self.html_response(WATCH_NOT_FOUND.to_vec(), 404, version),
                    None => text_response(404, "Not found\n"),
                },
            },
            Err(400) => text_response(400, "Bad request\n"),
            Err(403) => text_response(403, "Forbidden path\n"),
            Err(_) => text_response(500, "Could not read file\n"),
        }
    }

    fn file_response(
        &self,
        mut file: File,
        path: &Utf8Path,
        status: u16,
        version: Option<&str>,
    ) -> HttpResponse {
        if let Some(version) = version.filter(|_| {
            path.extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("html"))
        }) {
            let mut body = Vec::new();
            if file.read_to_end(&mut body).is_err() {
                return text_response(500, "Could not read file\n");
            }
            return self.html_response(body, status, version);
        }
        file_response(file, path, status)
    }

    fn html_response(&self, mut body: Vec<u8>, status: u16, version: &str) -> HttpResponse {
        let script = format!(
            "<script src=\"{}/_zellij/preview.js?version={version}\" defer></script>",
            encode(&self.prefix)
        );
        let position = body
            .windows(7)
            .rposition(|w| w.eq_ignore_ascii_case(b"</body>"))
            .unwrap_or(body.len());
        body.splice(position..position, script.bytes());
        bytes_response(status, body, "text/html; charset=utf-8")
    }
}

fn full(bytes: Bytes) -> Body {
    Full::new(bytes)
        .map_err(|never| match never {})
        .boxed_unsync()
}
fn bytes_response(status: u16, bytes: impl Into<Bytes>, content_type: &str) -> HttpResponse {
    let bytes = bytes.into();
    Response::builder()
        .status(status)
        .header("content-type", content_type)
        .header("content-length", bytes.len())
        .body(full(bytes))
        .expect("valid response headers")
}
fn text_response(status: u16, text: &'static str) -> HttpResponse {
    bytes_response(status, text, "text/plain; charset=utf-8")
}
fn file_response(file: File, path: &Utf8Path, status: u16) -> HttpResponse {
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
    let Ok(metadata) = file.metadata() else {
        return text_response(500, "Could not read file\n");
    };
    let stream = ReaderStream::new(tokio::fs::File::from_std(file)).map_ok(Frame::data);
    Response::builder()
        .status(status)
        .header("content-type", content_type)
        .header("content-length", metadata.len())
        .body(StreamBody::new(stream).boxed_unsync())
        .expect("valid response headers")
}
