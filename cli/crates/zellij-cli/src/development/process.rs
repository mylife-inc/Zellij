//! Own every Next descendant, including its forked development server.
use std::{
    io::{self, Write},
    process::{Command, ExitStatus, Stdio},
    thread,
    time::{Duration, Instant},
};
use zellij_core::process::{CommandExt, Running};

pub(super) struct Tree {
    process: Running,
    #[cfg(unix)]
    group: i32,
    #[cfg(windows)]
    job: Job,
    closed: bool,
}
impl Tree {
    pub fn spawn(command: &mut Command) -> io::Result<Self> {
        command.stdin(Stdio::piped());
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        let process = command.zel_spawn()?;
        #[cfg(windows)]
        let job = Job::attach(&process.child)?;
        let mut tree = Self {
            #[cfg(unix)]
            group: process.child.id() as i32,
            process,
            #[cfg(windows)]
            job,
            closed: false,
        };
        // The verified runner waits for this byte before it can fork Next.
        tree.process
            .child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(b"start\n")?;
        Ok(tree)
    }
    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        let status = self.process.try_wait()?;
        if status.is_some() {
            self.terminate_tree();
        }
        Ok(status)
    }
    fn terminate_tree(&mut self) {
        if self.closed {
            return;
        }
        #[cfg(unix)]
        unsafe {
            libc::kill(-self.group, libc::SIGKILL);
        }
        #[cfg(windows)]
        self.job.terminate();
        self.closed = true;
    }
    pub fn stop(&mut self) {
        if self.closed {
            return;
        }
        self.process.child.stdin.take();
        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline {
            // Intentional signal exits are not quiet-mode failures. Poll the
            // child directly and let cancel mark the capture as finished.
            if self.process.child.try_wait().ok().flatten().is_some() {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }
        self.terminate_tree();
        let _ = self.process.cancel();
    }
}
impl Drop for Tree {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(windows)]
struct Job(windows_sys::Win32::Foundation::HANDLE);
#[cfg(windows)]
impl Job {
    fn attach(child: &std::process::Child) -> io::Result<Self> {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::System::JobObjects::*;
        unsafe {
            let handle = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if handle.is_null() {
                return Err(io::Error::last_os_error());
            }
            let job = Self(handle);
            let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            if SetInformationJobObject(
                handle,
                JobObjectExtendedLimitInformation,
                &limits as *const _ as _,
                std::mem::size_of_val(&limits) as u32,
            ) == 0
                || AssignProcessToJobObject(handle, child.as_raw_handle() as _) == 0
            {
                return Err(io::Error::last_os_error());
            }
            Ok(job)
        }
    }
    fn terminate(&self) {
        unsafe {
            windows_sys::Win32::System::JobObjects::TerminateJobObject(self.0, 1);
        }
    }
}
#[cfg(windows)]
impl Drop for Job {
    fn drop(&mut self) {
        unsafe {
            windows_sys::Win32::Foundation::CloseHandle(self.0);
        }
    }
}
