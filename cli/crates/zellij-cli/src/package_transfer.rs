//! Cancellable package transfers. The network worker never owns cache files.
use reqwest::{Url, blocking::Client, redirect::Policy};
use std::{
    io::{Read, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};
use zellij_core::{Error, Result};

pub(crate) fn check_stop(stop: &AtomicBool, label: &str) -> Result<()> {
    if stop.load(Ordering::Acquire) {
        Err(Error::InvalidArgument(format!(
            "{label}: operation cancelled"
        )))
    } else {
        Ok(())
    }
}

// The worker owns only network state. The caller owns all files, can cancel even
// during a stalled connection/read, and never leaves a late writer in its cache.
pub(crate) fn download(
    url: Url,
    limit: u64,
    output: &mut impl Write,
    stop: &AtomicBool,
    label: &str,
) -> Result<()> {
    let fail = |message: String| Error::InvalidArgument(format!("{label}: {message}"));
    check_stop(stop, label)?;
    let origin = url.origin();
    let (sender, receiver) = mpsc::sync_channel(2);
    thread::spawn(move || {
        let result = (|| -> std::result::Result<(), String> {
            let client = Client::builder()
                .user_agent(concat!("zel/", env!("CARGO_PKG_VERSION")))
                .connect_timeout(Duration::from_secs(15))
                .timeout(Duration::from_secs(300))
                .redirect(Policy::custom(move |attempt| {
                    let url = attempt.url();
                    if attempt.previous().len() < 5
                        && url.username().is_empty()
                        && url.password().is_none()
                        && (url.scheme() == "https"
                            || url.scheme() == "http" && url.origin() == origin)
                    {
                        attempt.follow()
                    } else {
                        attempt.stop()
                    }
                }))
                .build()
                .map_err(|_| "could not initialize HTTPS client".to_owned())?;
            let mut response = client.get(url).send().map_err(|_| {
                "download failed; check connectivity or use a prepared cache with --offline"
                    .to_owned()
            })?;
            if !response.status().is_success() {
                return Err(format!(
                    "download returned HTTP {}; use a released CLI with matching package assets",
                    response.status()
                ));
            }
            if response.content_length().is_some_and(|size| size > limit) {
                return Err("download size limit exceeded".into());
            }
            let mut total = 0;
            loop {
                let mut bytes = vec![0; 65536];
                let n = response
                    .read(&mut bytes)
                    .map_err(|_| "package download was interrupted".to_owned())?;
                total += n as u64;
                if total > limit {
                    return Err("download size limit exceeded".into());
                }
                bytes.truncate(n);
                if sender.send(Ok(bytes)).is_err() || n == 0 {
                    return Ok(());
                }
            }
        })();
        if let Err(error) = result {
            let _ = sender.send(Err(error));
        }
    });
    loop {
        check_stop(stop, label)?;
        match receiver.recv_timeout(Duration::from_millis(50)) {
            Ok(Ok(bytes)) if bytes.is_empty() => return check_stop(stop, label),
            Ok(Ok(bytes)) => {
                check_stop(stop, label)?;
                output.write_all(&bytes)?;
            }
            Ok(Err(error)) => return Err(fail(error)),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err(fail("download ended unexpectedly".into()));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    #[test]
    fn pre_cancelled_transfer_never_contacts_the_server_or_writes() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let mut output = Vec::new();
        let result = download(
            format!("http://{}/package", listener.local_addr().unwrap())
                .parse()
                .unwrap(),
            32,
            &mut output,
            &AtomicBool::new(true),
            "test package",
        );
        assert!(result.unwrap_err().to_string().contains("cancelled"));
        assert!(output.is_empty());
        assert_eq!(
            listener.accept().unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
    }

    #[test]
    fn transfer_checks_status_declared_and_streamed_limits_and_truncation() {
        for (response, limit, success) in [
            ("HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\nabc", 3, true),
            ("HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\nabc", 2, false),
            ("HTTP/1.1 200 OK\r\nConnection: close\r\n\r\nabc", 2, false),
            (
                "HTTP/1.1 200 OK\r\nContent-Length: 10\r\n\r\nabc",
                32,
                false,
            ),
            (
                "HTTP/1.1 404 Not Found\r\nContent-Length: 3\r\n\r\nabc",
                32,
                false,
            ),
            (
                "HTTP/1.1 302 Found\r\nLocation: http://127.0.0.1:1/escape\r\nContent-Length: 0\r\n\r\n",
                32,
                false,
            ),
        ] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let url = format!("http://{}/package", listener.local_addr().unwrap())
                .parse()
                .unwrap();
            let server = thread::spawn(move || {
                let (mut socket, _) = listener.accept().unwrap();
                socket
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                let mut request = Vec::new();
                while !request.ends_with(b"\r\n\r\n") {
                    let mut byte = [0];
                    socket.read_exact(&mut byte).unwrap();
                    request.push(byte[0]);
                    assert!(request.len() < 8192);
                }
                socket.write_all(response.as_bytes()).unwrap();
            });
            let mut output = Vec::new();
            let result = download(
                url,
                limit,
                &mut output,
                &AtomicBool::new(false),
                "test package",
            );
            server.join().unwrap();
            assert_eq!(result.is_ok(), success, "{response}: {result:?}");
            assert!(output.len() <= limit as usize);
            if success {
                assert_eq!(output, b"abc");
            }
        }
    }
}
