use std::env;
use std::fmt;
use std::fmt::Write as _;
use std::fs;
use std::fs::File;
use std::io;
use std::os::unix;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::ansi::strip_csi;
use crate::console_writer::ConsoleWriter;
use crate::numbers::Numbers;
use crate::shell::shell_quote_args;
use crate::Number;
use std::io::Write;

pub struct RunLog {
    name: PathBuf,
    raw: PathBuf,
    last: Option<PathBuf>,
    file: File,
    console_writer: ConsoleWriter,
}

pub struct BothLogAndStderr<'a> {
    log: &'a mut RunLog,
}

impl RunLog {
    pub fn name(&self) -> &Path {
        &self.name
    }

    pub fn last(&self) -> Option<&Path> {
        self.last.as_deref()
    }

    pub fn open() -> RunLog {
        let home_dir = dirs::home_dir().expect("home_dir not found");
        let mut absh_logs_dir = home_dir.clone();
        absh_logs_dir.push(".absh/logs");
        let mut name = absh_logs_dir.clone();
        let id = format!(
            "{}",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );
        name.push(&id);

        fs::create_dir_all(&name).expect(&format!("failed to create dir {}", name.display()));

        let mut raw = name.clone();
        raw.push("raw");

        let mut log = name.clone();
        log.push("log");

        let file = File::create(&log).expect(&format!("failed to create {}", log.display()));

        #[cfg(unix)]
        let last = {
            let mut last = absh_logs_dir.clone();
            last.push("last");

            let _ = fs::remove_file(&last);
            unix::fs::symlink(name.file_name().unwrap(), &last).expect("symlink");
            Some(last)
        };
        #[cfg(not(unix))]
        let last = { None };

        RunLog {
            console_writer: ConsoleWriter::auto(),
            name,
            file,
            last,
            raw,
        }
    }

    pub fn both_log_and_stderr(&mut self) -> BothLogAndStderr {
        BothLogAndStderr { log: self }
    }

    pub fn log_only(&mut self) -> &mut File {
        &mut self.file
    }

    pub fn stderr_only(&mut self) -> &mut ConsoleWriter {
        &mut self.console_writer
    }

    pub fn write_raw<N: Number>(&mut self, durations: &[&Numbers<N>]) -> io::Result<()> {
        let mut content = String::new();
        fn join<N: Number>(r: &mut String, ds: &Numbers<N>) {
            for (i, d) in ds.iter().enumerate() {
                if i != 0 {
                    write!(r, " ").unwrap();
                }
                write!(r, "{}", d).unwrap();
            }
            write!(r, "\n").unwrap();
        }

        for d in durations {
            join(&mut content, d);
        }

        fs::write(&self.raw, content)
    }

    pub fn write_args(&mut self) -> anyhow::Result<()> {
        let mut args = shell_quote_args(env::args());
        args.push_str("\n");
        fs::write(self.name.join("args"), args)?;
        Ok(())
    }
}

impl fmt::Write for BothLogAndStderr<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.log.console_writer.write_str(s)?;
        let black_and_white = strip_csi(s);
        self.log
            .log_only()
            .write(black_and_white.as_bytes())
            .map_err(|_| fmt::Error)?;
        Ok(())
    }
}
