use std::fs;
use std::path::Path;

use anyhow::Context;

pub(crate) fn write_using_temp(
    path: impl AsRef<Path>,
    contents: impl AsRef<[u8]>,
) -> anyhow::Result<()> {
    let path = path.as_ref();
    let file_name = path
        .file_name()
        .context("no file name")?
        .to_str()
        .context("file name not UTF-8")?;
    let temp_path = path.with_file_name(format!(".{}!", file_name));
    fs::write(&temp_path, contents)?;
    fs::rename(temp_path, path)?;
    Ok(())
}
