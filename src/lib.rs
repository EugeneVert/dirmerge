use std::{
    fs,
    path::{Path, PathBuf},
};

pub enum Type {
    File,
    Dir,
}

pub struct Entry {
    pub path: PathBuf,
    pub _type: Type,
}

impl Entry {
    pub fn new(path: &Path) -> std::io::Result<Self> {
        let filetype = path.metadata()?.file_type();
        let _type = if filetype.is_dir() {
            Type::Dir
        } else {
            Type::File
        };
        Ok(Self {
            path: path.to_owned(),
            _type,
        })
    }

    pub fn mov(&self, to: &Path, overwrite: bool) -> std::io::Result<()> {
        match self._type {
            Type::File => {
                if to.exists() && !overwrite {
                    return Ok(());
                }
                if fs::rename(&self.path, to).is_err() {
                    // `to` is on a different mount point
                    fs::copy(&self.path, to)?;
                    fs::remove_file(&self.path)?;
                }
            }
            Type::Dir => {
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
                    ));
                }

                for dirent in read_dir(&self.path)? {
                    let to = to.join(dirent.path.file_name().unwrap());
                    dirent.mov(&to, overwrite)?;
                }
                fs::remove_dir(&self.path).ok();
            }
        }
        Ok(())
    }

    pub fn copy(&self, to: &Path, overwrite: bool) -> std::io::Result<()> {
        match self._type {
            Type::File => {
                if to.exists() && !overwrite {
                    return Ok(());
                }
                fs::copy(&self.path, to)?;
            }
            Type::Dir => {
                if !to.exists() {
                    fs::create_dir(to)?;
                }
                for dirent in read_dir(&self.path)? {
                    let to = to.join(dirent.path.file_name().unwrap());
                    dirent.copy(&to, overwrite)?;
                }
            }
        }
        Ok(())
    }
}

fn read_dir(path: &Path) -> std::io::Result<Vec<Entry>> {
    path.read_dir()?
        .map(|dirent| -> std::io::Result<Entry> {
            let dirent = dirent?;
            let path = dirent.path();
            Entry::new(&path)
        })
        .collect()
}
