//! Optional process-wide presentation policy. Frontends initialize it once,
//! before starting workers; library callers retain the existing default output.
//! This never selects a renderer, validation level, or publication behavior.
use std::{fmt, io::Write, sync::OnceLock};

#[derive(Clone, Copy, Debug, Default)]
pub struct Options {
    pub quiet: bool,
    pub verbose: bool,
    pub no_color: bool,
}

static OPTIONS: OnceLock<Options> = OnceLock::new();

pub fn initialize(options: Options) {
    OPTIONS
        .set(options)
        .expect("output policy initialized once");
}

pub fn options() -> Options {
    OPTIONS.get().copied().unwrap_or_default()
}

pub fn plain(text: &str) -> String {
    let mut writer = anstream::StripStream::new(Vec::new());
    writer.write_all(text.as_bytes()).expect("in-memory writer");
    String::from_utf8_lossy(&writer.into_inner()).into_owned()
}

pub fn emit(args: fmt::Arguments<'_>, stderr: bool) {
    let text = args.to_string();
    let text = if options().no_color {
        plain(&text)
    } else {
        text
    };
    if stderr {
        eprint!("{text}");
    } else {
        print!("{text}");
    }
}

/// Apply color preferences only to the child, never mutate this process's env.
pub fn configure_command(command: &mut std::process::Command) {
    if options().no_color {
        command
            .env("NO_COLOR", "1")
            .env("CLICOLOR", "0")
            .env_remove("CLICOLOR_FORCE")
            .env_remove("FORCE_COLOR");
    }
}

#[macro_export]
macro_rules! statusln {
    () => { $crate::statusln!(""); };
    ($($arg:tt)*) => {
        if !$crate::reporting::options().quiet {
            $crate::reporting::emit(format_args!("{}\n", format_args!($($arg)*)), false);
        }
    };
}

#[macro_export]
macro_rules! reportln {
    () => { $crate::reportln!(""); };
    ($($arg:tt)*) => {
        $crate::reporting::emit(format_args!("{}\n", format_args!($($arg)*)), false)
    };
}

#[macro_export]
macro_rules! diagnosticln {
    () => { $crate::diagnosticln!(""); };
    ($($arg:tt)*) => {
        $crate::reporting::emit(format_args!("{}\n", format_args!($($arg)*)), true)
    };
}

#[macro_export]
macro_rules! verbose {
    ($($arg:tt)*) => {
        if $crate::reporting::options().verbose {
            $crate::diagnosticln!("[verbose] {}", format_args!($($arg)*));
        }
    };
}
