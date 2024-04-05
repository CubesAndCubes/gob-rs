use std::string::FromUtf8Error;

pub fn string_from_bytes(bytes: &[u8]) -> Result<String, FromUtf8Error> {
    String::from_utf8(Vec::from(bytes))
}

macro_rules! slice {
    ($file:ident, $size:literal) => {{
        let mut slice_array = [0 as u8; $size];

        $file
            .read_exact(&mut slice_array)?;

        slice_array
    }};
}

pub(crate) use slice;

