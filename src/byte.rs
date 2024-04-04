pub fn string_from_bytes(bytes: &[u8]) -> String {
    String::from_utf8(Vec::from(bytes)).expect("Should be able to convert to String.")
}

macro_rules! slice {
    ($file:ident, $size:literal) => {{
        let mut slice_array = [0 as u8; $size];

        $file
            .read_exact(&mut slice_array)
            .expect("Should be able to read file.");

        slice_array
    }};
}

pub(crate) use slice;
