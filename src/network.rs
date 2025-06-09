#![warn(clippy::pedantic)]

use anyhow::Result;
use std::path::Path;

#[cfg(feature = "https")]
pub fn download_file(url: &str, target_location: &Path) -> Result<u64> {
    use std::{fs::File, io::Write};

    let response = reqwest::blocking::get(url)?;
    let content = response.bytes()?;

    let mut dest = File::create(target_location)?;
    dest.write_all(&content)?;

    Ok(1)
}

#[cfg(not(feature = "https"))]
pub fn download_file(_url: &str, _target_location: &Path) -> Result<u64> {
    use anyhow::bail;

    bail!("Attempted to download from a HTTPS source, but HTTPS feature not enabled.");
}

#[cfg(test)]
#[cfg(not(feature = "https"))]
mod tests {
    use super::*;

    #[test]
    #[should_panic(
        expected = "Attempted to download from a HTTPS source, but HTTPS feature not enabled."
    )]
    fn test_download_fail() {
        download_file("", Path::new("a")).unwrap();
    }
}

#[cfg(test)]
#[cfg(feature = "https")]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Read;
    use std::path::PathBuf;

    #[test]
    fn test_download_file_success() {
        // Uses gitignore as an example
        let url = "https://raw.githubusercontent.com/github/gitignore/main/Rust.gitignore";
        let target = PathBuf::from("test_download_file_success.txt");

        let result = download_file(url, &target);
        assert!(result.is_ok());
        assert!(target.exists());

        // Clean up
        let _ = fs::remove_file(&target);
    }

    #[test]
    fn test_download_file_invalid_url() {
        let url = "https://invalid.url/doesnotexist.txt";
        let target = PathBuf::from("test_download_file_invalid_url.txt");

        let result = download_file(url, &target);
        assert!(result.is_err());
        assert!(!target.exists());
    }

    #[test]
    fn test_download_file_invalid_path() {
        let url = "https://raw.githubusercontent.com/github/gitignore/main/Rust.gitignore";
        // Use an invalid path (directory does not exist)
        let target = PathBuf::from("/invalid_dir/test_download_file_invalid_path.txt");

        let result = download_file(url, &target);
        assert!(result.is_err_and(|x| x.is::<std::io::Error>()));
    }

    #[test]
    fn test_download_file_content() {
        let url = "https://raw.githubusercontent.com/github/gitignore/main/Rust.gitignore";
        let target = PathBuf::from("test_download_file_content.txt");

        let result = download_file(url, &target);
        assert!(result.is_ok());

        let mut file = fs::File::open(&target).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        assert!(contents.contains("target/")); // Known content in Rust.gitignore

        // Clean up
        let _ = fs::remove_file(&target);
    }
}
