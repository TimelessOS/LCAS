use std::{
    fs::{self, create_dir_all, rename},
    os::unix::fs::{PermissionsExt, symlink},
    path::Path,
};

use crate::artifacts::get_artifact;

mod artifacts;
mod compression;
mod hash;

#[derive(serde::Serialize, serde::Deserialize)]
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

pub fn create_store(store_path: &Path) -> Result<(), String> {
    if store_path.exists() {
        return Err("Store already exists! Make sure the directory doesn't exist, or you're operating on the correct directory.".to_string());
    }

    fs::create_dir_all(store_path).expect("Couldn't create repo!");
    fs::create_dir_all(store_path.join("chunks")).expect("Couldn't create repo!");
    fs::create_dir_all(store_path.join("manifests")).expect("Couldn't create repo!");
    fs::create_dir_all(store_path.join("artifacts")).expect("Couldn't create repo!");

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

        let root_path = input_dir.to_string_lossy().to_string();
        let path = entry.path().to_string_lossy().to_string();
        let raw = fs::read(&path).expect("Couldn't read file for chunking!");
        let compressed = compression::compress_file(&raw, 3);
        let hash = hash::hash(&raw);

        // Determine if the file is executable
        let is_executable = entry.path().metadata().unwrap().permissions().mode() & 0o111 != 0;

        // Save the chunk
        fs::write(chunk_dir.join(&hash), compressed).expect("Couldn't write chunk file!");

        files.push((path.replacen(&root_path, "", 1), hash, is_executable));
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

#[cfg(feature = "decoding")]
pub fn install_artifact(artifact_name: &String, store_path: &Path, repo_cache_path: &Path) {
    assert!(store_path.is_absolute(), "Store path must be absolute!");
    assert!(
        repo_cache_path.is_absolute(),
        "Repo cache path must be absolute!"
    );

    let repo_manifest_dir = repo_cache_path.join("manifests");
    let repo_artifacts_path = repo_cache_path.join("artifacts");

    let store_chunk_dir = store_path.join("chunks");
    let store_manifest_dir = store_path.join("manifests");
    let store_artifacts_path = store_path.join("artifacts");

    let manifest_hash =
        get_artifact(artifact_name, &repo_artifacts_path).expect("Artifact not found");

    let manifest: Manifest = serde_json::from_str(
        fs::read_to_string(repo_manifest_dir.join(&manifest_hash))
            .unwrap()
            .as_str(),
    )
    .unwrap();

    for (_path, hash, executable) in &manifest.files {
        // Install chunks
        install_chunk(&hash, store_path, repo_cache_path).unwrap();

        // Make sure it's executable if it needs to be
        if *executable {
            make_chunk_executable(&hash, store_path);
        }
    }

    // Seperate to ensure chunks have been installed prior to linked
    for (manifest_defined_path, hash, _executable) in &manifest.files {
        let manifest_defined_path = manifest_defined_path.trim_start_matches('/').to_string();

        let path = store_manifest_dir
            .join(&manifest_hash)
            .join(manifest_defined_path);

        if !path.parent().unwrap().exists() {
            create_dir_all(&path.parent().unwrap()).unwrap();
        }

        if !fs::exists(&path).unwrap() {
            symlink(store_chunk_dir.as_path().join(hash), path).unwrap();
        }
    }

    // Create a temporary symlink for atomic update
    let tmp_file_name = get_temp_file(None, &store_artifacts_path);

    let tmp_symlink = store_artifacts_path.join(&tmp_file_name);
    let final_symlink = store_artifacts_path.join(artifact_name);

    symlink(store_manifest_dir.join(&manifest_hash), &tmp_symlink).unwrap();
    rename(&tmp_symlink, &final_symlink).unwrap();
}

fn get_temp_file(potential: Option<u8>, dir: &Path) -> String {
    let potential = potential.unwrap_or_default();

    let file_name = format!(".tmp_{}", &potential);

    return if dir.join(&file_name).exists() {
        get_temp_file(Some(potential + 1), dir)
    } else {
        file_name
    };
}

fn make_chunk_executable(chunk_hash: &String, store_path: &Path) {
    let chunk_path = store_path.join("chunks").join(chunk_hash);

    // Read initial permissions first
    let mut perms = fs::metadata(&chunk_path).unwrap().permissions();

    // Probably not a good idea to hardcode this.
    perms.set_mode(0o755);
    fs::set_permissions(&chunk_path, perms).expect("Unable to set executable bit!");
}

#[cfg(feature = "decoding")]
fn install_chunk(
    chunk_hash: &String,
    store_path: &Path,
    repo_cache_path: &Path,
) -> Result<(), String> {
    use crate::compression::decompress_file;

    let repo_chunk_path = repo_cache_path.join("chunks").join(chunk_hash);
    let store_chunk_path = store_path.join("chunks").join(chunk_hash);

    // TODO: Network Functionality

    if repo_chunk_path.exists() {
        let mut repo_chunk = fs::read(repo_chunk_path).unwrap();
        let decompressed_chunk = decompress_file(&mut repo_chunk);

        // Verify hash
        let hash = hash::hash(&decompressed_chunk);
        if &hash != chunk_hash {
            panic!(
                "Unable to verify hash: Something has either been corrupted, or something malicous is happening!"
            )
        }

        fs::write(store_chunk_path, decompressed_chunk).unwrap();

        Ok(())
    } else {
        Err("Couldn't find chunk".to_string())
    }
}
