use anyhow::Result;
use rand;
use std::{
    fs::{create_dir_all, remove_dir_all},
    io,
    path::{Path, PathBuf},
};

/// A handler to easily create temporary directories and destroy them
#[derive(Debug, Clone)]
pub struct ContainerDir {
    path: Option<PathBuf>,
    destroy_on_drop: bool,
    pub id: String,
}

impl ContainerDir {
    fn gen_id() -> String {
        let u1: u128 = rand::random();
        format!("{:x}", u1)
    }

    /// Create a new directory at path <root_dir>/<random 32-byte id> and return the object
    pub fn new<P: AsRef<Path>>(root_dir: P) -> Result<Self> {
        let mut id = Self::gen_id();
        while root_dir.as_ref().join(&id).exists() {
            id = Self::gen_id();
        }
        create_dir_all(root_dir.as_ref().join(&id))?;
        return Ok(Self {
            path: Some(root_dir.as_ref().join(&id)),
            destroy_on_drop: false,
            id,
        });
    }

    /// Control whether the tempdir will be deleted when the object is dropped
    pub fn destroy_on_drop(&mut self, flag: bool) {
        self.destroy_on_drop = flag;
    }

    /// Forcefully remove the directory (the path becomes inaccessible after)
    pub fn close(&mut self) -> Result<()> {
        let err = io::Error::new(
            io::ErrorKind::InvalidInput,
            "Closing a ContainerDir which was already close",
        );
        remove_dir_all(self.path.as_ref().ok_or(err)?)?;
        Ok(())
    }
}

impl AsRef<Path> for ContainerDir {
    fn as_ref(&self) -> &Path {
        &self.path.as_ref().unwrap()
    }
}

impl Drop for ContainerDir {
    fn drop(&mut self) -> () {
        if self.destroy_on_drop {
            if let Some(p) = &self.path {
                let _ = remove_dir_all(p);
            }
        }
    }
}
