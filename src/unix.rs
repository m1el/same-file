use std::fs::{File, OpenOptions};
use std::hash::{Hash, Hasher};
use std::io;
use std::os::unix::fs::MetadataExt;
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
use std::path::Path;

/// Low level key structure
///
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Key {
    /// device number
    dev: u64,
    /// inode
    ino: u64,
}

#[derive(Debug)]
pub struct Handle {
    file: Option<File>,
    // If is_std is true, then we don't drop the corresponding File since it
    // will close the handle.
    is_std: bool,
    key: Key,
}

impl Drop for Handle {
    fn drop(&mut self) {
        if self.is_std {
            // unwrap() will not panic. Since we were able to open an
            // std stream successfully, then `file` is guaranteed to be Some()
            self.file.take().unwrap().into_raw_fd();
        }
    }
}

impl Eq for Handle {}

impl PartialEq for Handle {
    fn eq(&self, other: &Handle) -> bool {
        self.key == other.key
    }
}

impl AsRawFd for ::Handle {
    fn as_raw_fd(&self) -> RawFd {
        // unwrap() will not panic. Since we were able to open the
        // file successfully, then `file` is guaranteed to be Some()
        self.0.file.as_ref().take().unwrap().as_raw_fd()
    }
}

impl IntoRawFd for ::Handle {
    fn into_raw_fd(mut self) -> RawFd {
        // unwrap() will not panic. Since we were able to open the
        // file successfully, then `file` is guaranteed to be Some()
        self.0.file.take().unwrap().into_raw_fd()
    }
}

impl Hash for Handle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}

impl Handle {
    pub fn from_path<P: AsRef<Path>>(p: P) -> io::Result<Handle> {
        Handle::from_file(OpenOptions::new().read(true).open(p)?)
    }

    pub fn from_file(file: File) -> io::Result<Handle> {
        let md = file.metadata()?;
        Ok(Handle {
            file: Some(file),
            is_std: false,
            key: Key {
                dev: md.dev(),
                ino: md.ino(),
            },
        })
    }

    pub fn from_std(file: File) -> io::Result<Handle> {
        Handle::from_file(file).map(|mut h| {
            h.is_std = true;
            h
        })
    }

    pub fn stdin() -> io::Result<Handle> {
        Handle::from_std(unsafe { File::from_raw_fd(0) })
    }

    pub fn stdout() -> io::Result<Handle> {
        Handle::from_std(unsafe { File::from_raw_fd(1) })
    }

    pub fn stderr() -> io::Result<Handle> {
        Handle::from_std(unsafe { File::from_raw_fd(2) })
    }

    pub fn as_file(&self) -> &File {
        // unwrap() will not panic. Since we were able to open the
        // file successfully, then `file` is guaranteed to be Some()
        self.file.as_ref().take().unwrap()
    }

    pub fn as_file_mut(&mut self) -> &mut File {
        // unwrap() will not panic. Since we were able to open the
        // file successfully, then `file` is guaranteed to be Some()
        self.file.as_mut().take().unwrap()
    }

    pub fn dev(&self) -> u64 {
        self.key.dev
    }

    pub fn ino(&self) -> u64 {
        self.key.ino
    }

    pub fn as_key(&self) -> Option<Key> {
        Some(self.key.clone())
    }
}
