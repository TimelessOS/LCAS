use xxhash_rust::xxh3::xxh3_64;

// Hashes with xxh3
pub fn hash(input: &Vec<u8>) -> String {
    xxh3_64(&input).to_string()
}

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
    fn hash_stable() {
        let result = hash(&vec![1, 2, 3]);
        assert_eq!(result, "16991689376074199867");
    }
}
