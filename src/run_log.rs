use std::env;
use std::fmt;
use std::fmt::Write as _;
use std::fs;
use std::fs::File;
use std::os::unix;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::ansi::strip_csi;
use crate::console_writer::ConsoleWriter;
use crate::fs_util::write_using_temp;
use crate::math::numbers::Numbers;
use crate::maybe_strip_csi_writer::MaybeStripCsiWriter;
use crate::shell::shell_quote_args;

pub struct RunLog {
    name: PathBuf,
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
        }
    }

    pub fn both_log_and_stderr(&mut self) -> BothLogAndStderr {
        BothLogAndStderr { log: self }
    }

    pub fn log_only(&mut self) -> impl fmt::Write + '_ {
        MaybeStripCsiWriter {
            inner: &mut self.file,
            strip: true,
        }
    }

    pub fn stderr_only(&mut self) -> &mut ConsoleWriter {
        &mut self.console_writer
    }

    pub fn write_raw(&mut self, id: &str, durations: &[&Numbers]) -> anyhow::Result<()> {
        let mut content = String::new();
        fn join(r: &mut String, ds: &Numbers) -> anyhow::Result<()> {
            for (i, d) in ds.iter().enumerate() {
                if i != 0 {
                    write!(r, " ")?;
                }
                write!(r, "{}", d)?;
            }
            write!(r, "\n")?;
            Ok(())
        }

        for d in durations {
            join(&mut content, d)?;
        }

        write_using_temp(self.name.join(format!("raw-{}.txt", id)), content)?;
        Ok(())
    }

    pub fn write_graph(&mut self, graph: &str) -> anyhow::Result<()> {
        write_using_temp(self.name.join("graph.txt"), graph)?;
        write_using_temp(self.name.join("graph-bw.txt"), strip_csi(graph))?;

        let report_md = format!(
            "```\n{}\n```\n```\n{}```\n",
            Self::args_str(),
            strip_csi(graph),
        );
        write_using_temp(self.name.join("report.md"), report_md)?;
        Ok(())
    }

    fn args_str() -> String {
        shell_quote_args(env::args())
    }

    pub fn write_args(&mut self) -> anyhow::Result<()> {
        let mut args = Self::args_str();
        args.push_str("\n");
        write_using_temp(self.name.join("args.txt"), args)?;
        Ok(())
    }
}

impl fmt::Write for BothLogAndStderr<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.log.console_writer.write_str(s)?;
        write!(self.log.log_only(), "{}", s)?;
        Ok(())
    }
}
