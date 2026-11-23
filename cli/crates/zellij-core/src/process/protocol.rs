//! An owned child with a liveness pipe and bounded, private output.
//!
//! Protocol bytes never go through presentation-policy captures or the user's
//! console. Temporary files avoid pipe deadlocks, including inherited handles.
use super::Capture;
use std::{
    io::{self, Read},
    process::{Child, ChildStdin, Command, ExitStatus, Stdio},
    thread,
    time::{Duration, Instant},
};

pub struct ProtocolOutput {
    pub status: Option<ExitStatus>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

pub struct ProtocolChild {
    child: Child,
    input: Option<ChildStdin>,
    output: Capture,
    diagnostics: Capture,
    stdout_limit: u64,
    stderr_limit: u64,
    finished: bool,
}

fn read_bounded(capture: &mut Capture, limit: u64) -> io::Result<Vec<u8>> {
    let length = capture.file.metadata()?.len();
    if length < capture.position {
        return Err(io::Error::other("subprocess output was truncated"));
    }
    if length > limit {
        return Err(io::Error::other("subprocess output budget exceeded"));
    }
    let mut bytes = vec![0; (length - capture.position) as usize];
    capture.reader.read_exact(&mut bytes)?;
    capture.position = length;
    Ok(bytes)
}

impl ProtocolChild {
    /// The caller supplies a verified executable and its environment. No shell,
    /// reporting policy, arguments or environment variables are added here.
    pub fn spawn(command: &mut Command, stdout_limit: u64, stderr_limit: u64) -> io::Result<Self> {
        if stdout_limit == 0
            || stderr_limit == 0
            || stdout_limit > 8 * 1024 * 1024
            || stderr_limit > 8 * 1024 * 1024
        {
            return Err(io::Error::other("invalid subprocess output budget"));
        }
        let output = Capture::new()?;
        let diagnostics = Capture::new()?;
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(output.file.try_clone()?)
            .stderr(diagnostics.file.try_clone()?)
            .spawn()?;
        let input = child.stdin.take();
        Ok(Self {
            child,
            input,
            output,
            diagnostics,
            stdout_limit,
            stderr_limit,
            finished: false,
        })
    }

    pub fn id(&self) -> u32 {
        self.child.id()
    }

    /// Drain bounded deltas even after exit, without printing or waiting for EOF
    /// on stdout/stderr. The owner must keep polling and impose its own deadline.
    pub fn poll(&mut self) -> io::Result<ProtocolOutput> {
        let status = self.child.try_wait()?;
        self.finished |= status.is_some();
        Ok(ProtocolOutput {
            status,
            stdout: read_bounded(&mut self.output, self.stdout_limit)?,
            stderr: read_bounded(&mut self.diagnostics, self.stderr_limit)?,
        })
    }

    /// Close the liveness pipe, allow one second for cooperative shutdown, then
    /// kill and reap the direct child. This is not a descendant-tree killer.
    pub fn cancel(&mut self) -> io::Result<()> {
        self.input.take();
        let deadline = Instant::now() + Duration::from_secs(1);
        loop {
            if self.child.try_wait()?.is_some() {
                self.finished = true;
                return Ok(());
            }
            if Instant::now() >= deadline {
                if let Err(error) = self.child.kill()
                    && self.child.try_wait()?.is_none()
                {
                    return Err(error);
                }
                self.child.wait()?;
                self.finished = true;
                return Ok(());
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
}

impl Drop for ProtocolChild {
    fn drop(&mut self) {
        self.input.take();
        if !self.finished {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}
