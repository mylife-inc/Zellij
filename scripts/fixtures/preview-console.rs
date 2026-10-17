// Windows CI only: give zel its own console and deliver a real Ctrl-C to it.
// Node's child.kill('SIGINT') forcibly terminates Windows processes and cannot
// verify a console handler. This helper never signals the runner's console.
#[cfg(windows)]
fn main() {
    use std::{
        io::{self, BufRead},
        os::windows::process::CommandExt,
        process::{Command, Stdio},
        sync::mpsc,
        thread,
        time::{Duration, Instant},
    };
    #[link(name = "Kernel32")]
    unsafe extern "system" {
        fn FreeConsole() -> i32;
        fn AttachConsole(pid: u32) -> i32;
        fn SetConsoleCtrlHandler(
            handler: Option<unsafe extern "system" fn(u32) -> i32>,
            add: i32,
        ) -> i32;
        fn GenerateConsoleCtrlEvent(event: u32, group: u32) -> i32;
        fn GetStdHandle(kind: u32) -> *mut std::ffi::c_void;
        fn SetStdHandle(kind: u32, handle: *mut std::ffi::c_void) -> i32;
    }
    unsafe {
        SetConsoleCtrlHandler(None, 0);
    } // Do not inherit an ignored Ctrl-C from the CI parent.
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let mut child = Command::new(&args[0])
        .args(&args[1..])
        .creation_flags(0x00000010)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut output = child.stdout.take().unwrap();
    let mut errors = child.stderr.take().unwrap();
    let stdout = thread::spawn(move || io::copy(&mut output, &mut io::stdout()).unwrap());
    let stderr = thread::spawn(move || io::copy(&mut errors, &mut io::stderr()).unwrap());
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let _ = io::stdin().lock().lines().next();
        let _ = sender.send(());
    });
    let mut stopping = None;
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            stdout.join().unwrap();
            stderr.join().unwrap();
            std::process::exit(status.code().unwrap_or(1));
        }
        if stopping.is_none() && receiver.try_recv().is_ok() {
            // Save the piped handles: AttachConsole changes the process defaults.
            let kinds = [(-10i32) as u32, (-11i32) as u32, (-12i32) as u32];
            let ok = unsafe {
                let handles = kinds.map(|kind| GetStdHandle(kind));
                FreeConsole();
                let attached = AttachConsole(child.id());
                for (kind, handle) in kinds.into_iter().zip(handles) {
                    SetStdHandle(kind, handle);
                }
                attached != 0
                    && SetConsoleCtrlHandler(None, 1) != 0
                    && GenerateConsoleCtrlEvent(0, 0) != 0
            };
            if !ok {
                let _ = child.kill();
                panic!(
                    "could not send Ctrl-C to the private zel console: {}",
                    io::Error::last_os_error()
                );
            }
            stopping = Some(Instant::now());
        }
        if stopping.is_some_and(|since| since.elapsed() > Duration::from_secs(8)) {
            let _ = child.kill();
            panic!("zel did not exit after Ctrl-C");
        }
        thread::sleep(Duration::from_millis(20));
    }
}
#[cfg(not(windows))]
fn main() {
    panic!("this console helper is only used on Windows");
}
