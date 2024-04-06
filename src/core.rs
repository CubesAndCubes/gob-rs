use std::{
    fs,
    io::{Error, ErrorKind, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

use crate::byte;

/// An object representing a GOB archive in memory.
/// 
/// Instances of `Gob` hold a [`Vec`] of [`File`], which represent
/// the individual files contained within the archive.
/// 
/// # Examples
/// 
/// Creates a new object from parsing a GOB file at a given [`Path`]:
/// 
/// ```
/// use std::path::Path;
/// use gob_rs::core::Gob;
/// 
/// let gob = Gob::from_file(Path::new("/path/to/gob.GOB"))?;
/// ```
/// 
/// Creates a new object from parsing a directory, structured like
/// a GOB archive, at a given [`Path`]:
/// 
/// ```
/// use std::path::Path;
/// use gob_rs::core::Gob;
/// 
/// let gob = Gob::from_directory(Path::new("/path/to/gob"))?;
/// ```
/// 
/// Gets the file count of the archive:
/// 
/// ```
/// let file_count = gob.files.len();
/// ```
/// 
/// 
pub struct Gob {
    /// A [`Vec`] of [`File`], representing the files contained within
    /// the archive object.
    pub files: Vec<File>,
}

impl Gob {
    fn get_files_from_directory(
        files: &mut Vec<File>,
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

                let file = File::new(data, filepath)?;

                files.push(file);
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
    /// ```
    /// use std::path::Path;
    /// use gob_rs::core::Gob;
    /// 
    /// let gob = Gob::from_directory(Path::new("/path/to/gob"))?;
    /// ```
    pub fn from_directory(path: &Path) -> std::io::Result<Self> {
        if !path.is_dir() {
            return Err(Error::new(ErrorKind::InvalidInput, "Path is not a directory."));
        }

        let mut directory = fs::read_dir(path)?;
        
        let mut files: Vec<File> = Vec::new();

        Self::get_files_from_directory(&mut files, &mut directory, None)?;

        Ok(Self { files })
    }

    /// Creates a new [`Gob`] object from a given [`Path`] to a GOB archive file.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use std::path::Path;
    /// use gob_rs::core::Gob;
    /// 
    /// let gob = Gob::from_file(Path::new("/path/to/gob.GOB"))?;
    /// ```
    pub fn from_file(path: &Path) -> std::io::Result<Self> {
        if !path.is_file() {
            return Err(Error::new(ErrorKind::InvalidInput, "Path is not a file."));
        }

        let mut file = fs::File::open(path)?;

        file.seek(SeekFrom::Start(0))?;

        let signature = &byte::slice!(file, 4);

        if signature != b"GOB " {
            return Err(Error::new(ErrorKind::InvalidData, "Bad signature in header of GOB file."));
        }

        let version = u32::from_le_bytes(byte::slice!(file, 4));

        if version != 0x14 {
            return Err(Error::new(ErrorKind::InvalidData, "Bad version in header of GOB file."));
        }

        let body_offset = u32::from_le_bytes(byte::slice!(file, 4)) as u64;

        file.seek(SeekFrom::Start(body_offset))?;

        let file_count = u32::from_le_bytes(byte::slice!(file, 4));

        let mut file_definitions: Vec<FileDefinition> = Vec::new();

        for _ in 0..file_count {
            let offset = u32::from_le_bytes(byte::slice!(file, 4)) as usize;

            let size = u32::from_le_bytes(byte::slice!(file, 4)) as usize;

            let filepath = match byte::string_from_bytes(&byte::slice!(file, 128)) {
                Ok(filepath) => filepath,
                Err(_) => {
                    return Err(Error::new(ErrorKind::InvalidData, "Bad string encoding."));
                }
            };

            let filepath = PathBuf::from(
                filepath.trim_matches(char::from(0))
            );

            file_definitions.push(FileDefinition {
                offset,
                size,
                filepath,
            });
        }

        let mut files: Vec<File> = Vec::new();

        for file_definition in file_definitions {
            file.seek(SeekFrom::Start(file_definition.offset as u64))?;

            let mut data: Vec<u8> = vec![0; file_definition.size];

            file.read_exact(&mut data)?;

            let file = File::new(data, file_definition.filepath)?;

            files.push(file);
        }

        Ok(Self { files })
    }
}

struct FileDefinition {
    offset: usize,
    size: usize,
    filepath: PathBuf,
}

/// An object representing a file within a [`Gob`] archive.
///
/// # Examples
/// 
/// Creating a GOB archive file:
/// 
/// ```
/// use std::path::PathBuf; 
/// use gob_rs::core::File;
/// 
/// let archive_file = File::new(
///     "GOB".as_bytes().to_vec(),
///     PathBuf::from("foo.bar")
/// )?;
/// ```
/// 
/// Creating a GOB archive file from a real file:
/// 
/// ```
/// use std::path::PathBuf; 
/// use gob_rs::core::File;
/// 
/// let mut real_file = fs::File::open(&path)?;
/// 
/// let mut data: Vec<u8> = Vec::new();
/// 
/// real_file.read_to_end(&mut data)?;
/// 
/// let archive_file = File::new(
///     data,
///     PathBuf::from("foo.bar"),
/// )?;
/// ```
///  
/// # Limitations
///
/// Due to some strict memory definitions in the structure of
/// GOB archives, filepaths may at most be 128 ASCII characters
/// (or 128 bytes) long.
pub struct File {
    /// The bytes of the file.
    pub data: Vec<u8>,

    /// The relative path of the file within the archive.
    pub filepath: PathBuf,
}

impl File {
    /// Creates a new [`File`] object from a given [`Vec`] of [`u8`] (bytes)
    pub fn new(data: Vec<u8>, filepath: PathBuf) -> std::io::Result<Self> {
        if filepath.as_os_str().as_encoded_bytes().len() > 128 {
            return Err(Error::new(ErrorKind::InvalidInput, "File path is longer than 128 bytes."));
        }

        Ok(Self { data, filepath })
    }
}
