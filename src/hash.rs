use xxhash_rust::xxh3::xxh3_64;

// Hashes with xxh3
fn hash(input: &Vec<u8>) -> String {
    xxh3_64(&input).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = hash(&vec![1, 2, 3]);
        assert_eq!(result, "16991689376074199867");
    }
}
