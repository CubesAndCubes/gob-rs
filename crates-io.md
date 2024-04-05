A Rust library for parsing and constructing archives of the LucasArts GOB format.

# Examples

## Parsing GOB file

```rs
use std::path::Path;
use gob_rs::core::Gob;

let gob = Gob::from_file(Path::new("/path/to/gob.GOB"));
```

## Parsing GOB-like directory

```rs
use std::path::Path;
use gob_rs::core::Gob;

let gob = Gob::from_directory(Path::new("/path/to/gob/"));
```

## Getting The File Count

```rs
let file_count = gob.files.len();
```