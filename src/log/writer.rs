use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::util::path::create_log_path;

pub(crate) fn open_write_log() -> Result<(PathBuf, fs::File), io::Error> {
    let log_path = create_log_path()?;
    let log = open_log_path(&log_path)?;
    Ok((log_path, log))
}

fn open_log_path(path: &Path) -> Result<fs::File, io::Error> {
    fs::OpenOptions::new().create(true).append(true).open(path)
}
