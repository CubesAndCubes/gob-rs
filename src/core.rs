use std::{
    fs,
    io::{Read, Seek, SeekFrom},
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
/// Creates a new object from parsing a GOB file at a given [`PathBuf`]:
/// 
/// ```
/// use std::path::PathBuf;
/// use gob_rs::core::Gob;
/// 
/// let gob = Gob::from(PathBuf::from("/path/to/gob.GOB"));
/// ```
/// 
/// Creates a new object from parsing a directory, structured like
/// a GOB archive, at a given [`PathBuf`]:
/// 
/// ```
/// use gob_rs::core::Gob;
/// 
/// let gob = Gob::from(PathBuf::from("/path/to/gob/"));
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

            let root = root.unwrap_or(
                path.parent()
                    .expect("Should be able to get parent directory"),
            );

            if path.is_file() {
                let mut file = fs::File::open(&path)?;

                let mut data: Vec<u8> = Vec::new();

                file.read_to_end(&mut data)?;

                let filepath: PathBuf = path
                    .strip_prefix(root)
                    .expect("Should be able to get relative path")
                    .into();

                let file = File::new(data, filepath);

                files.push(file);
            } else if path.is_dir() {
                let mut directory = path.read_dir()?;

                Self::get_files_from_directory(files, &mut directory, Some(root))?;
            } else {
                panic!("Path is neither file nor directory: {}", path.display());
            }
        }

        Ok(())
    }

    fn from_directory(directory: &mut fs::ReadDir) -> Self {
        let mut files: Vec<File> = Vec::new();

        Self::get_files_from_directory(&mut files, directory, None)
            .expect("Should be able to get files from directory");

        Self { files }
    }

    fn from_file(file: &mut fs::File) -> Self {
        file.seek(SeekFrom::Start(0))
            .expect("Should be able to seek to start.");

        let signature = &byte::slice!(file, 4);

        if signature != b"GOB " {
            panic!("Bad signature in header of gob file.");
        }

        let version = u32::from_le_bytes(byte::slice!(file, 4));

        if version != 0x14 {
            panic!("Bad version {version} for gob file.");
        }

        let body_offset = u32::from_le_bytes(byte::slice!(file, 4)) as u64;

        file.seek(SeekFrom::Start(body_offset)).expect(&format!(
            "Should be able to seek to body offset ({body_offset})."
        ));

        let file_count = u32::from_le_bytes(byte::slice!(file, 4));

        let mut file_definitions: Vec<FileDefinition> = Vec::new();

        for _ in 0..file_count {
            let offset = u32::from_le_bytes(byte::slice!(file, 4)) as usize;

            let size = u32::from_le_bytes(byte::slice!(file, 4)) as usize;

            let filepath = PathBuf::from(
                byte::string_from_bytes(&byte::slice!(file, 128)).trim_matches(char::from(0)),
            );

            file_definitions.push(FileDefinition {
                offset,
                size,
                filepath,
            });
        }

        let mut files: Vec<File> = Vec::new();

        for file_definition in file_definitions {
            file.seek(SeekFrom::Start(file_definition.offset as u64))
                .expect(&format!(
                    "Should be able to seek to file offset ({}).",
                    file_definition.offset
                ));

            let mut data: Vec<u8> = vec![0; file_definition.size];

            file.read_exact(&mut data)
                .expect("Should be able to read file data.");

            let file = File::new(data, file_definition.filepath);

            files.push(file);
        }

        Self { files }
    }
}

impl From<&mut fs::File> for Gob {
    fn from(file: &mut fs::File) -> Self {
        Self::from_file(file)
    }
}

impl From<&mut fs::ReadDir> for Gob {
    fn from(directory: &mut fs::ReadDir) -> Self {
        Self::from_directory(directory)
    }
}

impl From<PathBuf> for Gob {
    fn from(path: PathBuf) -> Self {
        if path.is_file() {
            let mut file = fs::File::open(path).expect("Should be able to open file.");

            Self::from_file(&mut file)
        } else if path.is_dir() {
            let mut directory = fs::read_dir(path).expect("Should be able to read directory.");

            Self::from_directory(&mut directory)
        } else {
            panic!("Path is neither file nor directory: {}", path.display());
        }
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
/// Creating a file:
/// 
/// ```
/// use gob_rs::core::File;
/// 
/// let file = File::new(
///     "GOB".as_bytes().to_vec(),
///     PathBuf::from("foo/bar")
/// );
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

    /// The relative filepath of the file within the archive.
    pub filepath: PathBuf,
}

impl File {
    pub fn new(data: Vec<u8>, filepath: PathBuf) -> Self {
        if filepath.as_os_str().as_encoded_bytes().len() > 128 {
            panic!("File path is longer than 128 bytes: {}", filepath.display());
        }

        Self { data, filepath }
    }
}
