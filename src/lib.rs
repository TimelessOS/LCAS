use std::{fs, path::Path};

mod artifacts;
mod compression;
mod hash;

#[derive(serde::Serialize)]
pub struct Manifest {
    pub files: Vec<(String, String, bool)>, // (path, hash, executable)
    pub format: u8,
}

// Creates a repo, or returns an Error if a repo already exists
#[cfg(feature = "encoding")]
pub fn create_repo(repo_dir: &Path) -> Result<(), String> {
    if repo_dir.exists() {
        return Err("Repo already exists! Make sure the directory doesn't exist, or you're operating on the correct directory.".to_string());
    }

    fs::create_dir_all(repo_dir).expect("Couldn't create repo!");
    fs::create_dir_all(repo_dir.join("chunks")).expect("Couldn't create repo!");
    fs::create_dir_all(repo_dir.join("manifests")).expect("Couldn't create repo!");

    Ok(())
}

// Creates a manifest and it's associated chunks from a dir structure
#[cfg(feature = "encoding")]
pub fn build(input_dir: &Path, repo_dir: &Path, artifact_name: &String) -> Result<String, String> {
    use walkdir::WalkDir;

    // List of all files used by the new manifest
    let mut files = Vec::new();
    // Define some directories
    let chunk_dir = repo_dir.join("chunks");
    let manifest_dir = repo_dir.join("manifests");
    let artifacts_file_path = repo_dir.join("artifacts");

    // Walk the input directory and process files
    for entry in WalkDir::new(input_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        use std::os::unix::fs::PermissionsExt;

        let path = entry.path().to_string_lossy().to_string();
        let raw = fs::read(&path).expect("Couldn't read file for chunking!");
        let compressed = compression::compress_file(&raw, 3);
        let hash = hash::hash(&raw);

        // Determine if the file is executable
        let is_executable = entry.path().metadata().unwrap().permissions().mode() & 0o111 != 0;

        // Save the chunk
        fs::write(chunk_dir.join(&hash), compressed).expect("Couldn't write chunk file!");

        files.push((path, hash, is_executable));
    }

    let manifest = Manifest {
        format: 1,
        files: files,
    };

    let manifest_hash = hash::hash_manifest(&manifest.files);

    // Write the manifest to the repo directory
    fs::write(
        manifest_dir.join(&manifest_hash),
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .expect("Couldn't write manifest!");

    artifacts::add_artifact(
        artifact_name.clone(),
        manifest_hash.clone(),
        &artifacts_file_path,
    );

    Ok(manifest_hash)
}
