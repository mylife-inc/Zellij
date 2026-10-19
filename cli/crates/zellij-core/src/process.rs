//! Child progress handling. Use temporary files instead of undrained pipes:
//! even large output cannot block a child, and descendants holding inherited
//! descriptors do not delay cancellation/reaping. Files are removed on drop.
use crate::reporting;
use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom, Write},
    process::{Child, Command, ExitStatus, Stdio},
    thread,
    time::Duration,
};

const FAILURE_TAIL: u64 = 64 * 1024;

struct Capture {
    file: File,
    reader: File,
    position: u64,
}
impl Capture {
    fn new() -> io::Result<Self> {
        // Open independent cursors before unlinking. File::try_clone shares the
        // writer's cursor; Windows seek_read also changes it. Reopening avoids
        // corrupting concurrent writes on every OS. No path survives startup.
        let named = tempfile::NamedTempFile::new()?;
        let reader = named.reopen()?;
        let (file, path) = named.into_parts();
        path.close()?;
        Ok(Self {
            file,
            reader,
            position: 0,
        })
    }

    fn drain(&mut self, writer: &mut impl Write) -> io::Result<()> {
        // Snapshot length: a surviving descendant cannot keep this loop alive.
        self.drain_until(writer, self.file.metadata()?.len())
    }
    fn drain_until(&mut self, writer: &mut impl Write, length: u64) -> io::Result<()> {
        let mut buffer = [0; 8192];
        while self.position < length {
            let count = (length - self.position).min(buffer.len() as u64) as usize;
            let read = self.reader.read(&mut buffer[..count])?;
            if read == 0 {
                break;
            }
            self.position += read as u64;
            writer.write_all(&buffer[..read])?;
        }
        writer.flush()
    }
    fn failure(&mut self) -> io::Result<()> {
        let length = self.file.metadata()?.len();
        self.position = length.saturating_sub(FAILURE_TAIL);
        self.reader.seek(SeekFrom::Start(self.position))?;
        if self.position > 0 {
            crate::diagnosticln!("[earlier subprocess output omitted; last 64 KiB follows]");
        }
        if reporting::options().no_color {
            self.drain_until(&mut anstream::StripStream::new(io::stderr()), length)
        } else {
            self.drain_until(&mut io::stderr(), length)
        }
    }
}

/// Owns the child and its logs through normal exit, cancellation, and IO errors.
pub struct Running {
    pub child: Child,
    captures: Option<(Capture, Capture)>,
    stdout: anstream::StripStream<io::Stdout>,
    stderr: anstream::StripStream<io::Stderr>,
    finished: bool,
}
impl Running {
    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        let status = self.child.try_wait()?;
        if !self.finished {
            if let Some((out, err)) = &mut self.captures {
                if reporting::options().quiet {
                    if status.is_some_and(|status| !status.success()) {
                        out.failure()?;
                        err.failure()?;
                    }
                } else {
                    out.drain(&mut self.stdout)?;
                    err.drain(&mut self.stderr)?;
                }
            }
            self.finished = status.is_some();
        }
        Ok(status)
    }
    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        loop {
            if let Some(status) = self.try_wait()? {
                return Ok(status);
            }
            thread::sleep(Duration::from_millis(20));
        }
    }
    pub fn kill(&mut self) -> io::Result<()> {
        self.child.kill()
    }
}
impl Drop for Running {
    fn drop(&mut self) {
        if !self.finished {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

pub trait CommandExt {
    fn zel_spawn(&mut self) -> io::Result<Running>;
    fn zel_status(&mut self) -> io::Result<ExitStatus>;
    /// Commands whose output is deliberately hidden even without --quiet.
    fn zel_spawn_silent(&mut self) -> io::Result<Running>;
}
impl Running {
    fn start(command: &mut Command, silent: bool) -> io::Result<Self> {
        reporting::configure_command(command);
        crate::verbose!("Starting external tool (arguments and environment omitted).");
        let options = reporting::options();
        let captures = if silent {
            command.stdout(Stdio::null()).stderr(Stdio::null());
            None
        } else if options.quiet || options.no_color {
            let out = Capture::new()?;
            let err = Capture::new()?;
            command
                .stdout(out.file.try_clone()?)
                .stderr(err.file.try_clone()?);
            Some((out, err))
        } else {
            None
        };
        Ok(Self {
            child: command.spawn()?,
            captures,
            stdout: anstream::StripStream::new(io::stdout()),
            stderr: anstream::StripStream::new(io::stderr()),
            finished: false,
        })
    }
}
impl CommandExt for Command {
    fn zel_spawn(&mut self) -> io::Result<Running> {
        Running::start(self, false)
    }
    fn zel_spawn_silent(&mut self) -> io::Result<Running> {
        Running::start(self, true)
    }
    fn zel_status(&mut self) -> io::Result<ExitStatus> {
        self.zel_spawn()?.wait()
    }
}
