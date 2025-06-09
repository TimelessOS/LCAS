#![warn(clippy::pedantic)]

use std::{fs, path::Path};

fn read_artifacts_file(artifacts_file_path: &Path) -> Vec<(String, String)> {
    if !artifacts_file_path.exists() {
        return Vec::new();
    }

    let artifacts_file =
        fs::read_to_string(artifacts_file_path).expect("Couldn't open artifacts file");

    let mut lines: Vec<(String, String)> = Vec::new();

    for line in artifacts_file.lines() {
        let parts = line.split_once(':').expect("Malformed artifacts file");
        lines.push((parts.0.to_string(), parts.1.to_string()));
    }

    lines
}

#[cfg(feature = "decoding")]
pub fn get_artifact(artifact_name: &String, artifacts_file_path: &Path) -> Option<String> {
    let artifacts = read_artifacts_file(artifacts_file_path);

    artifacts
        .iter()
        .find(|this_artifact| {
            dbg!(this_artifact);
            &this_artifact.0 == artifact_name
        })
        .map(|artifact| artifact.1.clone())
}

#[cfg(feature = "encoding")]
pub fn add_artifact(artifact_name: String, manifest_hash: String, artifacts_file_path: &Path) {
    let mut artifacts: Vec<(String, String)> = read_artifacts_file(artifacts_file_path)
        .iter()
        .filter(|artifact| artifact.0 != artifact_name)
        .map(|artifact| (artifact.0.clone(), artifact.1.clone()))
        .collect();

    artifacts.push((artifact_name, manifest_hash));

    fs::write(artifacts_file_path, serialize_artifacts(artifacts))
        .expect("Couldn't write artifacts file");
}

#[cfg(feature = "encoding")]
fn serialize_artifacts(artifacts: Vec<(String, String)>) -> String {
    use std::fmt::Write;

    let mut string = String::new();

    for artifact in artifacts {
        writeln!(&mut string, "{}:{}", artifact.0, artifact.1).unwrap();
    }

    string
}

#[cfg(test)]
mod tests {
    use std::env::temp_dir;

    use super::*;

    #[test]
    #[cfg(all(feature = "encoding", feature = "decoding"))]
    fn simple_all() {
        let artifacts = vec![
            ("test1".to_string(), 1.to_string()),
            ("test2".to_string(), 2.to_string()),
            ("test3".to_string(), 3.to_string()),
        ];

        let artifact_file_path = temp_dir().join("LCAS_test_artifact_simple_all.test");

        for (artifact_name, manifest_hash) in &artifacts {
            add_artifact(
                artifact_name.to_string(),
                manifest_hash.to_string(),
                &artifact_file_path,
            );
        }

        assert_eq!(&read_artifacts_file(&artifact_file_path), &artifacts);

        assert_eq!(
            get_artifact(&"test1".to_string(), &artifact_file_path).unwrap(),
            1.to_string()
        );
    }

    #[test]
    #[cfg(all(feature = "encoding", feature = "decoding"))]
    fn get_artifact_not_found() {
        let artifact_file_path = temp_dir().join("LCAS_test_artifact_not_found.test");
        // Ensure file is empty
        fs::write(&artifact_file_path, "").unwrap();
        assert_eq!(
            get_artifact(&"nonexistent".to_string(), &artifact_file_path),
            None
        );
    }

    #[test]
    #[cfg(feature = "encoding")]
    fn add_artifact_overwrites_existing() {
        let artifact_file_path = temp_dir().join("LCAS_test_artifact_overwrite.test");
        add_artifact(
            "artifact".to_string(),
            "hash1".to_string(),
            &artifact_file_path,
        );
        add_artifact(
            "artifact".to_string(),
            "hash2".to_string(),
            &artifact_file_path,
        );
        let artifacts = read_artifacts_file(&artifact_file_path);
        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0], ("artifact".to_string(), "hash2".to_string()));
    }

    #[test]
    fn read_artifacts_file_nonexistent() {
        let artifact_file_path = temp_dir().join("LCAS_test_artifact_nonexistent.test");
        // Ensure file does not exist
        let _ = fs::remove_file(&artifact_file_path);
        let artifacts = read_artifacts_file(&artifact_file_path);
        assert!(artifacts.is_empty());
    }

    #[test]
    #[should_panic(expected = "Malformed artifacts file")]
    fn read_artifacts_file_malformed() {
        let artifact_file_path = temp_dir().join("LCAS_test_artifact_malformed.test");
        fs::write(&artifact_file_path, "bad_line_without_colon\n").unwrap();
        let _ = read_artifacts_file(&artifact_file_path);
    }

    #[test]
    #[cfg(feature = "encoding")]
    fn simple_serialize_artifacts() {
        let artifacts = vec![
            ("test1".to_string(), 1.to_string()),
            ("test2".to_string(), 2.to_string()),
            ("test3".to_string(), 3.to_string()),
        ];
        assert_eq!(
            serialize_artifacts(artifacts),
            "test1:1\ntest2:2\ntest3:3\n"
        );
    }
}
