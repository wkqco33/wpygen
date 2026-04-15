use std::fs;
use std::path::Path;

use crate::error::Error;
use crate::models::ProjectSpec;
use crate::templates;

pub fn create_project(target_dir: &Path, spec: &ProjectSpec, force: bool) -> Result<usize, Error> {
    ensure_target_dir(target_dir, force)?;

    let files = templates::render_project(spec);
    for file in &files {
        let path = target_dir.join(&file.relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| Error::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        fs::write(&path, &file.contents).map_err(|source| Error::Io {
            path: path.clone(),
            source,
        })?;
    }

    Ok(files.len())
}

fn ensure_target_dir(target_dir: &Path, force: bool) -> Result<(), Error> {
    if target_dir.exists() {
        if !target_dir.is_dir() {
            return Err(Error::TargetPathIsFile(target_dir.to_path_buf()));
        }
        let is_empty = fs::read_dir(target_dir)
            .map_err(|source| Error::Io {
                path: target_dir.to_path_buf(),
                source,
            })?
            .next()
            .is_none();
        if !is_empty && !force {
            return Err(Error::TargetDirectoryNotEmpty(target_dir.to_path_buf()));
        }
        return Ok(());
    }

    fs::create_dir_all(target_dir).map_err(|source| Error::Io {
        path: target_dir.to_path_buf(),
        source,
    })
}
