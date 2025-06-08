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
    let mut artifacts: Vec<(String, String)> = read_artifacts_file(&artifacts_file_path)
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
    let mut string = String::new();

    for artifact in artifacts {
        string += &format!("{}:{}\n", artifact.0, artifact.1);
    }

    string
}
