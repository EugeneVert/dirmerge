use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;

pub enum Type {
    File,
    Dir,
}

pub struct File {
    pub path: PathBuf,
}

pub struct Dir {
    pub path: PathBuf,
}

pub trait Entry {
    fn mov(&self, to: &Path, overwrite: bool) -> Result<()>;
    fn copy(&self, to: &Path, overwrite: bool) -> Result<()>;
    fn kind(&self) -> Type;
    fn path(&self) -> &Path;
}

impl Entry for File {
    fn mov(&self, to: &Path, overwrite: bool) -> Result<()> {
        if to.exists() && !overwrite {
            return Ok(());
        }
        if fs::rename(&self.path, to).is_err() {
            // `to` is on a different mount point
            fs::copy(&self.path, to)?;
            fs::remove_file(&self.path)?;
        }
        Ok(())
    }

    fn copy(&self, to: &Path, overwrite: bool) -> Result<()> {
        if to.exists() && !overwrite {
            return Ok(());
        }
        fs::copy(&self.path, to)?;
        Ok(())
    }

    fn kind(&self) -> Type {
        Type::File
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Entry for Dir {
    fn mov(&self, to: &Path, overwrite: bool) -> Result<()> {
        if !to.exists() {
            if fs::rename(&self.path, to).is_err() {
                // `to` is on a different mount point
                self.copy(to, overwrite)?;
                fs::remove_dir_all(&self.path)?;
            }
            return Ok(());
        }

        let to_filetype = to.metadata()?.file_type();
        if to_filetype.is_file() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "File with directory name already exists",
            ).into());
        }

        for dirent in read_dir(&self.path)? {
            dbg!(dirent.path().display());
            let to = to.join(dirent.path().file_name().unwrap());
            dirent.mov(&to, overwrite)?;
        }
        fs::remove_dir(&self.path).ok();
        Ok(())
    }

    fn copy(&self, to: &Path, overwrite: bool) -> Result<()> {
        if !to.exists() {
            fs::create_dir(to)?;
        }
        for dirent in read_dir(&self.path)? {
            let to = to.join(dirent.path().file_name().unwrap());
            dirent.copy(&to, overwrite)?;
        }
        Ok(())
    }

    fn kind(&self) -> Type {
        Type::Dir
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

fn read_dir(path: &Path) -> Result<Vec<Box<dyn Entry>>> {
    path.read_dir()?
        .map(|dirent| -> Result<Box<dyn Entry>> {
            let dirent = dirent?;
            let path = dirent.path();
            let filetype = path.metadata()?.file_type();
            if filetype.is_dir() {
                Ok(Box::new(Dir { path }))
            } else {
                Ok(Box::new(File { path }))
            }
        })
        .collect()
}
