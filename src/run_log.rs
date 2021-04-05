use std::fmt::Write as _;
use std::fs;
use std::fs::File;
use std::io;
use std::os::unix;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::Duration;

pub struct RunLog {
    name: PathBuf,
    raw: PathBuf,
    last: Option<PathBuf>,
    file: File,
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
            name,
            file,
            last,
            raw,
        }
    }

    pub fn write_raw(&mut self, a: &[Duration], b: &[Duration]) -> io::Result<()> {
        let mut content = String::new();
        fn join(r: &mut String, ds: &[Duration]) {
            for (i, d) in ds.iter().enumerate() {
                if i != 0 {
                    write!(r, " ").unwrap();
                }
                write!(r, "{}", d).unwrap();
            }
            write!(r, "\n").unwrap();
        }

        join(&mut content, a);
        join(&mut content, b);

        fs::write(&self.raw, content)
    }
}

impl io::Write for RunLog {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}
