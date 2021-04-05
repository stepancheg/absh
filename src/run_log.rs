use std::fs;
use std::fs::File;
use std::io;
use std::os::unix;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;

pub struct RunLog {
    name: PathBuf,
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
        fs::create_dir_all(&absh_logs_dir).expect(&format!(
            "failed to create dir {}",
            &absh_logs_dir.display()
        ));
        let mut name = absh_logs_dir.clone();
        name.push(format!(
            "{}",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        ));
        let file = File::create(&name).expect(&format!("failed to create {}", name.display()));

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

        RunLog { name, file, last }
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
