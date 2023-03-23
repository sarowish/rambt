use anyhow::{bail, Result};
use std::path::PathBuf;

const APP_NAME: &str = "rambt";

fn get_data_dir() -> Result<PathBuf> {
    let path = match dirs::data_local_dir() {
        Some(path) => path.join(APP_NAME),
        None => bail!("Couldn't find local data directory"),
    };

    if !path.exists() {
        std::fs::create_dir_all(&path)?;
    }

    Ok(path)
}

pub fn get_database_path() -> Result<PathBuf> {
    Ok(get_data_dir()?.join("ratings.db"))
}
