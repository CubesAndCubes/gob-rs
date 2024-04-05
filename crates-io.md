# Examples

## Parsing GOB file

```rs
use std::path::PathBuf;
use gob_rs::core::Gob;

let gob = Gob::from(PathBuf::from("/path/to/gob.GOB"));
```

## Parsing GOB-like directory

```rs
use gob_rs::core::Gob;

let gob = Gob::from(PathBuf::from("/path/to/gob/"));
```