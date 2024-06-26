//! Core module of the library

use std::{
    collections::HashMap, fs, io::{Error, ErrorKind, Read, Seek, SeekFrom}, path::{Path, PathBuf}
};

use crate::byte;

/// An object representing a GOB archive.
/// 
/// Instances of `Gob` hold a [`GobMap`], representing
/// the structure of the archive.
/// 
/// # Examples
/// 
/// Creates a new object from parsing a GOB archive file at a given [`Path`]:
/// 
/// ```no_run
/// use std::path::Path;
/// use gob_rs::core::Gob;
/// 
/// fn main() -> std::io::Result<()> {
///     let gob = Gob::from_file(Path::new("/path/to/gob.GOB"))?;
/// 
///     Ok(())
/// }
/// ```
/// 
/// Creates a new object from parsing a directory, structured like
/// a GOB archive, at a given [`Path`]:
/// 
/// ```no_run
/// use std::path::Path;
/// use gob_rs::core::Gob;
/// 
/// fn main() -> std::io::Result<()> {
///     let gob = Gob::from_directory(Path::new("/path/to/gob"))?;
/// 
///     Ok(())
/// }
/// ```
/// 
/// Gets the file count of the archive:
/// 
/// ```
/// use std::path::PathBuf;
/// use gob_rs::core::Gob;
/// 
/// let mut gob = Gob::new();
/// 
/// gob.files.insert(
///     PathBuf::from("foo.bar"),
///     b"foobar".to_vec(),
/// );
/// 
/// gob.files.insert(
///     PathBuf::from("fizz.buzz"),
///     b"fizzbuzz".to_vec(),
/// );
/// 
/// let file_count = gob.files.len();
/// 
/// assert_eq!(file_count, 2);
/// ```
/// 
/// Iterates over the archive:
/// 
/// ```
/// use std::path::PathBuf;
/// use gob_rs::core::Gob;
/// 
/// let mut gob = Gob::new();
/// 
/// gob.files.insert(
///     PathBuf::from("foo.bar"),
///     b"foobar".to_vec(),
/// );
/// 
/// gob.files.insert(
///     PathBuf::from("fizz.buzz"),
///     b"fizzbuzz".to_vec(),
/// );
/// 
/// for (filepath, data) in &gob.files {
///     println!("path: {} data: {:?}", filepath.display(), data);
/// }
/// ```
pub struct Gob {
    /// A [`GobMap`], representing the structure of the archive.
    pub files: GobMap,
}

impl Gob {
    fn get_files_from_directory(
        files: &mut GobMap,
        directory: &mut fs::ReadDir,
        root: Option<&Path>,
    ) -> std::io::Result<()> {
        for item in directory {
            let item = item?;

            let path = item.path();

            let root = match root {
                Some(root) => root,
                None => match path.parent() {
                    Some(root) => root,
                    None => {
                        return Err(Error::new(ErrorKind::Other, "Unable to get parent directory from path."));
                    }
                }
            };

            if path.is_file() {
                let mut file = fs::File::open(&path)?;

                let mut data: Vec<u8> = Vec::new();

                file.read_to_end(&mut data)?;

                let filepath: PathBuf = path
                    .strip_prefix(root)
                    .expect("Should be able to get relative path")
                    .into();

                files.insert(filepath, data);
            } else if path.is_dir() {
                let mut directory = path.read_dir()?;

                Self::get_files_from_directory(files, &mut directory, Some(root))?;
            } else {
                return Err(Error::new(ErrorKind::InvalidInput, "Path is neither file nor directory."));
            }
        }

        Ok(())
    }

    /// Creates a new [`Gob`] object from a given [`Path`] to a directory,
    /// structured like a GOB archive.
    /// 
    /// # Examples
    /// ```no_run
    /// use std::path::Path;
    /// use gob_rs::core::Gob;
    /// 
    /// fn main() -> std::io::Result<()> {
    ///     let gob = Gob::from_directory(Path::new("/path/to/gob"))?;
    /// 
    ///     Ok(())
    /// }
    /// ```
    pub fn from_directory(path: &Path) -> std::io::Result<Self> {
        if !path.is_dir() {
            return Err(Error::new(ErrorKind::InvalidInput, "Path is not a directory."));
        }

        let mut directory = fs::read_dir(path)?;
        
        let mut files = GobMap::new();

        Self::get_files_from_directory(&mut files, &mut directory, None)?;

        Ok(Self { files })
    }

    const SIGNATURE: &'static [u8; 4] = b"GOB ";

    const VERSION: u32 = 0x14;

    /// Creates a new [`Gob`] object from a given [`Path`] to a GOB archive file.
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// use std::path::Path;
    /// use gob_rs::core::Gob;
    /// 
    /// fn main() -> std::io::Result<()> {
    ///     let gob = Gob::from_file(Path::new("/path/to/gob.GOB"))?;
    /// 
    ///     Ok(())
    /// }
    /// ```
    pub fn from_file(path: &Path) -> std::io::Result<Self> {
        if !path.is_file() {
            return Err(Error::new(ErrorKind::InvalidInput, "Path is not a file."));
        }

        let mut file = fs::File::open(path)?;

        file.seek(SeekFrom::Start(0))?;

        let signature = &byte::slice!(file, 4);

        if signature != Self::SIGNATURE {
            return Err(Error::new(ErrorKind::InvalidData, "Bad signature in header of GOB file."));
        }

        let version = u32::from_le_bytes(byte::slice!(file, 4));

        if version != Self::VERSION {
            return Err(Error::new(ErrorKind::InvalidData, "Bad version in header of GOB file."));
        }

        let body_offset = u32::from_le_bytes(byte::slice!(file, 4)) as u64;

        file.seek(SeekFrom::Start(body_offset))?;

        let file_count = u32::from_le_bytes(byte::slice!(file, 4));

        let mut file_definitions: Vec<FileDefinition> = Vec::new();

        for _ in 0..file_count {
            let offset = u32::from_le_bytes(byte::slice!(file, 4)) as usize;

            let size = u32::from_le_bytes(byte::slice!(file, 4)) as usize;

            let filepath_bytes = byte::slice!(file, 128);

            let filepath_end = filepath_bytes.iter().position(|&n| n == 0).unwrap_or(128);

            let filepath = match byte::string_from_bytes(&filepath_bytes[..filepath_end]) {
                Ok(filepath) => filepath,
                Err(_) => {
                    return Err(Error::new(ErrorKind::InvalidData, format!("Cannot convert following bytes to string: {filepath_bytes:?}")));
                }
            };

            let filepath = PathBuf::from(filepath);

            file_definitions.push(FileDefinition {
                offset,
                size,
                filepath,
            });
        }

        let mut files = GobMap::new();

        for file_definition in file_definitions {
            file.seek(SeekFrom::Start(file_definition.offset as u64))?;

            let mut data: Vec<u8> = vec![0; file_definition.size];

            file.read_exact(&mut data)?;

            files.insert(file_definition.filepath, data);
        }

        Ok(Self { files })
    }

    /// Generates the data (bytes) for a GOB file representing the current archive object.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use std::path::PathBuf;
    /// use gob_rs::core::Gob;
    /// 
    /// let mut gob = Gob::new();
    /// 
    /// gob.files.insert(
    ///     PathBuf::from("foo.bar"),
    ///     b"foobar".to_vec(),
    /// );
    /// 
    /// gob.files.insert(
    ///     PathBuf::from("fizz.buzz"),
    ///     b"fizzbuzz".to_vec(),
    /// );
    /// 
    /// let data = gob.as_bytes();
    /// 
    /// assert_eq!(&data[..4], Vec::from(b"GOB "));
    /// 
    /// assert_eq!(&data[4..8], Vec::from(0x14u32.to_le_bytes()));
    /// 
    /// assert_eq!(&data[8..12], Vec::from(12u32.to_le_bytes()));
    /// 
    /// assert_eq!(&data[12..16], Vec::from(2u32.to_le_bytes()));
    /// ```
    pub fn as_bytes(self) -> Result<Vec<u8>, String> {
        let mut bytes: Vec<u8> = Vec::new();

        bytes.extend(Self::SIGNATURE);

        bytes.extend(&Self::VERSION.to_le_bytes());

        let body_offset: u32 = 12;

        bytes.extend(&body_offset.to_le_bytes());

        let file_count = self.files.len() as u32;

        bytes.extend(&file_count.to_le_bytes());

        let mut file_data_offset: u32 = 16 + 136 * file_count;

        for (filepath, file_data) in &self.files {
            bytes.extend(&file_data_offset.to_le_bytes());

            let size = file_data.len() as u32;

            file_data_offset += size;

            bytes.extend(&size.to_le_bytes());

            let filepath_bytes = filepath.as_os_str().as_encoded_bytes();

            if filepath_bytes.len() > 128 {
                return Err(format!("Filepath is longer than 128 bytes: {}", filepath.display()))
            }

            bytes.extend(filepath_bytes);

            bytes.extend(vec![0; 128 - filepath_bytes.len()]);
        }

        for (_, file_data) in &self.files {
            bytes.extend(file_data);
        }

        Ok(bytes)
    }

    /// Creates a new [`Gob`] object.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use gob_rs::core::Gob;
    /// 
    /// let gob = Gob::new();
    /// ```
    pub fn new() -> Self {
        let files = GobMap::new();

        Self {
            files,
        }
    }
}

impl From<GobMap> for Gob {
    fn from(files: GobMap) -> Self {
        Self {
            files
        }
    }
}


struct FileDefinition {
    offset: usize,
    size: usize,
    filepath: PathBuf,
}

/// A [`HashMap`] keyed by [`PathBuf`] containing [`Vec`] of [`u8`] (bytes),
/// representing the structure of a GOB archive.
/// 
/// # Examples
/// 
/// Creating object and inserting file:
/// ```
/// use std::path::PathBuf;
/// use gob_rs::core::GobMap;
/// 
/// let mut files = GobMap::new();
/// 
/// files.insert(
///     PathBuf::from("foo.bar"),
///     b"fizzbuzz".to_vec(),
/// );
/// 
/// assert_eq!(
///     files.get(&PathBuf::from("foo.bar")),
///     Some(&b"fizzbuzz".to_vec()),
/// );
/// ```
pub type GobMap = HashMap<PathBuf, Vec<u8>>;