use std::io;

// Compresses with ZSTD
fn compress_file(input: &Vec<u8>, level: i32) -> Vec<u8> {
    let mut buf = Vec::new();

    let mut encoder = zstd::stream::Encoder::new(&mut buf, level).unwrap();
    let mut cursor = std::io::Cursor::new(&input);
    io::copy(&mut cursor, &mut encoder).unwrap();
    encoder.finish().unwrap();

    buf
}

// Decompresses with ZSTD
fn decompress_file(input: &mut Vec<u8>) -> Vec<u8> {
    use std::io::Read;

    let mut cursor = std::io::Cursor::new(input);
    let mut decoder = zstd::stream::Decoder::new(&mut cursor).unwrap();
    let mut buf = Vec::new();
    decoder.read_to_end(&mut buf).unwrap();

    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_as_initial() {
        let original = vec![1, 2, 3, 4, 5];
        let compressed = compress_file(&original, 3);
        let decompressed = decompress_file(&mut compressed.clone());
        assert_eq!(original, decompressed);
    }
}
