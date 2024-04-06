A Rust library for parsing and constructing archives of the LucasArts GOB format.

# Examples

## Parsing GOB File

```rs
use std::path::Path;
use gob_rs::core::Gob;

fn main() -> std::io::Result<()> {
    let gob = Gob::from_file(Path::new("/path/to/gob.GOB"))?;

    Ok(())
}
```

## Parsing GOB-Like* Directory

*\*That is, a directory structured like a GOB archive.*

```rs
use std::path::Path;
use gob_rs::core::Gob;

fn main() -> std::io::Result<()> {
    let gob = Gob::from_directory(Path::new("/path/to/gob"))?;

    Ok(())
}
```

## Getting The File Count

```rs
let file_count = gob.files.len();
```