# gob-rs

A Rust library for parsing and constructing archives of the LucasArts GOB format.

## Examples

### Parsing GOB File

```rs
use std::path::Path;
use gob_rs::core::Gob;

let gob = Gob::from_file(Path::new("/path/to/gob.GOB"))?;
```

### Parsing GOB-Like Directory

```rs
use std::path::Path;
use gob_rs::core::Gob;

let gob = Gob::from_directory(Path::new("/path/to/gob/"))?;
```

### Getting The File Count

```rs
let file_count = gob.files.len();
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

### Limitations

One major limitation that arises due to the strict memory definitions of the file format is that the relative paths of files within a GOB archive may at most be 128 ASCII characters (or 128 bytes) long.

Another limitation is that due to the 32-Bit architecture of the format, GOB archives can at most reach a size of about 4 GB before breaking due to being unable to reference data offset past the 32-Bit limit.

## License

This library is dual-licensed under the [MIT license](LICENSE-MIT) and [Apache License, Version 2.0](LICENSE-APACHE).
