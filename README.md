# gob-rs

A Rust library for parsing and constructing archives of the LucasArts GOB format.

## Examples

### Parsing GOB file

```rs
use std::path::PathBuf;
use gob_rs::core::Gob;

let gob = Gob::from(PathBuf::from("/path/to/gob.GOB"));
```

### Parsing GOB-like directory

```rs
use gob_rs::core::Gob;

let gob = Gob::from(PathBuf::from("/path/to/gob/"));
```

## Specification

GOB files are used by LucasArts games built on the Sith engine as an archive format for storing game files.

They are encoded in the little-endian format.

The file structure can be abstracted as follows:

```rs
Gob {
    header: Header,
    body: Body,
}

Header {
    signature: 4 bytes, // must be "GOB "
    version: 4 bytes, // must be 0x14 -> 20
    body_offset: 4 bytes, // usually 0xC -> 12; byte address where body starts
}

Body {
    file_count: 4 bytes, // amount of files in archive
    files: [File; file_count], // file definitions
    ...file_data, // data of files; makes up remainder, thus size is variable
}

File {
    offset: 4 bytes, // byte address where file data starts
    size: 4 bytes, // size of file data in bytes
    filepath: 128 bytes, // path of file within archive
}
```