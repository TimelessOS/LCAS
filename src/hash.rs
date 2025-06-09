#![warn(clippy::pedantic)]

use xxhash_rust::xxh3::xxh3_64;

// Hashes with xxh3
#[cfg(any(feature = "decoding", feature = "encoding"))]
pub fn hash(input: &[u8]) -> String {
    xxh3_64(input).to_string()
}

// Converts the manifest to a string, and then hashes it
#[cfg(feature = "encoding")]
pub fn hash_manifest(input: &Vec<(String, String, bool)>) -> String {
    // Not a true hash. Just a temporary place to dump data, which can then be hashed
    let mut current_hash = "".to_string();

    for (file, hash, executable) in input {
        current_hash += file;
        current_hash += hash;
        // No clue why rust wants it as a str, but this works.
        current_hash += executable.to_string().as_str();
    }

    xxh3_64(current_hash.as_bytes()).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(any(feature = "decoding", feature = "encoding"))]
    fn hash_stable() {
        let result = hash(&vec![1, 2, 3]);
        assert_eq!(result, "16991689376074199867");
    }

    #[test]
    #[cfg(any(feature = "decoding", feature = "encoding"))]
    fn hash_empty_vec() {
        let result = hash(&vec![]);
        assert_eq!(result, "3244421341483603138");
    }

    #[test]
    #[cfg(feature = "encoding")]
    fn hash_manifest_stable() {
        let manifest = vec![
            ("file1.txt".to_string(), "hash1".to_string(), false),
            ("file2.txt".to_string(), "hash2".to_string(), true),
        ];
        let result = hash_manifest(&manifest);
        assert_eq!(result, "1259801786371591190");
    }

    #[test]
    #[cfg(feature = "encoding")]
    fn hash_manifest_empty() {
        let manifest: Vec<(String, String, bool)> = vec![];
        let result = hash_manifest(&manifest);
        assert_eq!(result, "3244421341483603138");
    }

    #[test]
    #[cfg(feature = "encoding")]
    fn hash_manifest_single_entry() {
        let manifest = vec![("main.rs".to_string(), "abc123".to_string(), true)];
        let result = hash_manifest(&manifest);
        assert_eq!(result, "9815591975043689442");
    }
}
