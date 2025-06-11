#![warn(clippy::pedantic)]

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use std::fs::create_dir_all;
use std::{
    fs,
    path::{Path, PathBuf},
};

mod artifacts;
mod compression;
mod hash;
mod network;

#[derive(serde::Serialize, serde::Deserialize)]
struct Manifest {
    files: Vec<(String, String, bool)>, // (path, hash, executable)
    format: u8,
}

pub enum RepoType {
    Local,
    Https,
}

/// Only used for decoding.
#[cfg(feature = "decoding")]
pub struct Store {
    // The "Upstream" RepoType
    pub kind: RepoType,
    /// Path to the Repo, can be a network location depending on `kind`
    pub repos: Vec<String>,
    /// The cache directory is currently only used with networked based `RepoType`, but may be used a future release.
    pub cache_path: PathBuf,
    /// The directory where all installed artifacts will be under, alongside the CAS System itself.
    pub path: PathBuf,
}

/// Attempts to create the repo and it's associated directories.
///
/// This operation will create all parent directories if they do not exist.
/// Any errors encountered during directory creation are ignored.
///
/// # Arguments
///
/// * `repo_dir` - The base directory for the repo.
///
/// # Side Effects
///
/// Creates the directory structure on the filesystem if it does not already exist.
///
/// # Errors
///
/// Any errors returned by `fs::create_dir_all` are ignored.
#[cfg(feature = "encoding")]
pub fn create_repo(repo_dir: &Path) -> Result<()> {
    if repo_dir.exists() {
        bail!("Already exists! {}", repo_dir.display())
    }

    () = fs::create_dir_all(repo_dir.join("chunks"))?;
    () = fs::create_dir_all(repo_dir.join("manifests"))?;

    Ok(())
}

/// Attempts to create the Store and it's associated directories.
///
/// This operation will create all parent directories if they do not exist.
/// Any errors encountered during directory creation are ignored.
///
/// This function should only be used when the Store does not already exist, and is *not* to create the `Store` struct.
///
/// # Arguments
///
/// * `store` - The correlated Store struct.
///
/// # Side Effects
///
/// Creates the directory structure on the filesystem if it does not already exist.
///
/// # Errors
///
/// Any errors returned by `fs::create_dir_all` are ignored.
#[cfg(feature = "decoding")]
pub fn create_store(store: &Store) -> Result<()> {
    if store.path.exists() {
        bail!("Store already exists! Make sure the directory doesn't exist, or you're operating on the correct directory.".to_string());
    }

    fs::create_dir_all(store.path.join("chunks"))?;
    fs::create_dir_all(store.path.join("manifests"))?;
    fs::create_dir_all(store.path.join("artifacts"))?;

    Ok(())
}

/// Creates a manifest and its associated chunks from a directory structure, and saves it into the list of artifacts.
///
/// This function walks the given input directory, compresses and hashes each file, and stores the resulting chunks and manifest in the repository.
/// The manifest is then registered as an artifact under the specified name.
///
/// # Arguments
///
/// * `input_dir` - The directory containing files to be added to the artifact.
/// * `repo_dir` - The base directory of the repository where chunks and manifests will be stored.
/// * `artifact_name` - The name under which the artifact will be registered.
///
/// # Returns
///
/// Returns the hash of the created manifest as a `String`.
///
/// # Errors
///
/// Returns an error if:
/// - Any file in the input directory cannot be read.
/// - Any chunk or manifest cannot be written to the repository.
/// - Any other I/O or processing error occurs during the build process.
#[cfg(feature = "encoding")]
pub fn build(input_dir: &PathBuf, repo_dir: &Path, artifact_name: &str) -> Result<String> {
    use std::os::unix::fs::PermissionsExt;
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
        let root_path = input_dir.to_string_lossy().to_string();
        let path = entry.path().to_string_lossy().to_string();
        let raw = fs::read(&path)
            .map_err(|x| anyhow::anyhow!("Couldn't read {:?} with error {}", &path, x))?;
        let compressed = compression::compress_file(&raw, 3);
        let hash = hash::hash(&raw);

        // Determine if the file is executable
        let is_executable = entry.path().metadata()?.permissions().mode() & 0o111 != 0;

        // Save the chunk
        fs::write(chunk_dir.join(&hash), compressed)?;

        files.push((path.replacen(&root_path, "", 1), hash, is_executable));
    }

    let manifest = Manifest { format: 1, files };

    let manifest_hash = hash::hash_manifest(&manifest.files);

    // Write the manifest to the repo directory
    fs::write(
        manifest_dir.join(&manifest_hash),
        serde_json::to_string_pretty(&manifest)?,
    )?;

    artifacts::add_artifact(
        artifact_name.to_string(),
        manifest_hash.to_string(),
        &artifacts_file_path,
    );

    Ok(manifest_hash)
}

/// Installs an Artifact by name.
///
/// # Arguments
///
/// * `artifact_name` - The name of the artifact to install.
/// * `store` - The correlated Store struct.
///
/// # Errors
/// Returns an error if the artifact does not exist, or if any file operations fail.
#[cfg(feature = "decoding")]
pub fn install_artifact(artifact_name: &String, store: &Store) -> Result<()> {
    use crate::artifacts::get_artifact;
    use anyhow::anyhow;
    use std::fs::{create_dir_all, rename};
    use std::os::unix::fs::symlink;

    let store_chunk_dir = store.path.join("chunks");
    let store_manifest_dir = store.path.join("manifests");
    let store_artifacts_path = store.path.join("artifacts");

    let manifest_hash = get_artifact(
        artifact_name,
        &resolve_repo_path(store, &"artifacts".to_string())?,
    )
    .ok_or_else(|| anyhow!("Tried to get a manifest that didn't exist"))?;

    let manifest: Manifest = serde_json::from_str(
        fs::read_to_string(resolve_repo_path(
            store,
            &format!("manifests/{manifest_hash}"),
        )?)?
        .as_str(),
    )?;

    for (_path, hash, executable) in &manifest.files {
        // Install chunks
        install_chunk(hash, store)?;

        // Make sure it's executable if it needs to be
        if *executable {
            // TODO: test this
            make_chunk_executable(hash, &store.path)?;
        }
    }

    // Seperate to ensure chunks have been installed prior to linked
    for (manifest_defined_path, hash, _executable) in &manifest.files {
        let manifest_defined_path = manifest_defined_path.trim_start_matches('/').to_string();

        let path = store_manifest_dir
            .join(&manifest_hash)
            .join(manifest_defined_path);

        if !path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Failed to get parent directory"))?
            .exists()
        {
            create_dir_all(
                path.parent()
                    .ok_or_else(|| anyhow::anyhow!("Failed to get parent directory"))?,
            )?;
        }

        if !&path.try_exists()? {
            symlink(store_chunk_dir.as_path().join(hash), path)?;
        }
    }

    // Create a temporary symlink for atomic update
    let tmp_file_name = get_temp_file(None, &store_artifacts_path);

    let tmp_symlink = store_artifacts_path.join(&tmp_file_name);
    let final_symlink = store_artifacts_path.join(artifact_name);

    symlink(store_manifest_dir.join(&manifest_hash), &tmp_symlink)?;
    rename(&tmp_symlink, &final_symlink)?;

    Ok(())
}

#[cfg(feature = "decoding")]
fn resolve_repo_path(store: &Store, path: &String) -> Result<PathBuf> {
    use core::error;

    if store.cache_path.join(path).exists() {
        return Ok(store.cache_path.join(path));
    }

    let joined_path = store.cache_path.join(path);
    let parent = joined_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Failed to get parent directory"))?;
    create_dir_all(parent)?;

    // List of all errors accumulated in the next for loop.
    let mut error_list = vec![];

    for repo in &store.repos {
        let result = match store.kind {
            RepoType::Https => {
                network::download_file(repo, &store.cache_path.join(path)).map(|_| ())
            }
            RepoType::Local => {
                fs::copy(PathBuf::from(&repo).join(path), store.cache_path.join(path))
                    .map(|_| ())
                    .map_err(|e| anyhow::anyhow!(e))
            }
        };

        if result.is_ok() {
            return Ok(store.cache_path.join(path));
        }

        // If error, just add to `error_list`` and continue to next repo, do not return error
        error_list.push(result.unwrap_err());
    }

    Err(anyhow::anyhow!("{:?}", error_list))
}

#[cfg(feature = "decoding")]
fn get_temp_file(potential: Option<u8>, dir: &Path) -> PathBuf {
    let potential = potential.unwrap_or_default();

    let file_name = format!(".tmp_{potential}");

    // Overflow protection, if the potential number is too high, we clear the oldest old temp file and replace it.
    if potential > u8::MAX - 1 {
        // Find the most recently modified temp file and remove it to prevent overflow
        if let Ok(entries) = std::fs::read_dir(dir) {
            let oldest_temp = entries
                .flatten()
                .filter(|f| {
                    f.file_type().map(|ft| ft.is_file()).unwrap_or(false)
                        && f.file_name().to_string_lossy().starts_with(".tmp_")
                })
                .min_by_key(|f| f.metadata().and_then(|m| m.modified()).ok())
                .map(|f| f.path())
                .unwrap();

            fs::remove_file(&oldest_temp).unwrap_or_else(|x| {
                panic!("Failed to remove old temp symlink: {x}");
            });

            return oldest_temp;
        }
    }

    // Check if the file already exists, if it does, increment the potential number
    if dir.join(&file_name).exists() {
        get_temp_file(Some(potential + 1), dir)
    } else {
        dir.join(&file_name)
    }
}

#[cfg(feature = "decoding")]
fn make_chunk_executable(chunk_hash: &String, store_path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let chunk_path = store_path.join("chunks").join(chunk_hash);

    // Read initial permissions first
    let mut perms = fs::metadata(&chunk_path)?.permissions();

    // Probably not a good idea to hardcode this, but it's a sensible default.
    perms.set_mode(0o755);
    fs::set_permissions(&chunk_path, perms)?;
    Ok(())
}

#[cfg(feature = "decoding")]
fn install_chunk(chunk_hash: &String, store: &Store) -> Result<()> {
    use crate::compression::decompress_file;

    let repo_chunk_path = resolve_repo_path(store, &format!("chunks/{chunk_hash}"))
        .with_context(|| format!("Couldn't find chunk {chunk_hash}"))?;
    let store_chunk_path = store.path.join("chunks").join(chunk_hash);

    let mut repo_chunk = fs::read(repo_chunk_path)?;
    let decompressed_chunk = decompress_file(&mut repo_chunk);

    // Verify hash
    let hash = hash::hash(&decompressed_chunk);
    if &hash != chunk_hash {
        bail!("Unable to verify hash")
    }

    fs::write(store_chunk_path, decompressed_chunk)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{env::temp_dir, fs::remove_dir_all};

    use std::fs;
    use std::fs::File;
    use std::os::unix::fs::PermissionsExt;

    #[cfg(feature = "encoding")]
    use crate::create_repo;

    #[cfg(feature = "decoding")]
    use crate::{RepoType, Store, create_store, resolve_repo_path};

    #[test]
    #[cfg(all(feature = "encoding", feature = "decoding"))]
    fn test_create_store() {
        let _ = create_test_store("basic");
    }

    #[cfg(all(feature = "encoding", feature = "decoding"))]
    fn create_test_store(test_name: &str) -> Store {
        let repo = temp_dir().join(format!("lcas_testing_repo_{test_name}"));
        let cache = temp_dir().join(format!("lcas_testing_cache_{test_name}"));
        let store_path = temp_dir().join(format!("lcas_testing_store_{test_name}"));

        let _ = remove_dir_all(&repo);
        let _ = remove_dir_all(&cache);
        let _ = remove_dir_all(&store_path);

        let store = Store {
            cache_path: cache,
            kind: RepoType::Local,
            path: store_path,
            repos: vec![repo.to_string_lossy().to_string()],
        };
        create_repo(&repo).unwrap();
        create_store(&store).unwrap();

        store
    }

    #[test]
    #[cfg(all(feature = "encoding", feature = "decoding"))]
    fn test_store_to_cache_empty() {
        let store = create_test_store("store_to_cache_empty");

        assert!(resolve_repo_path(&store, &"manifests/undefined".to_string()).is_err());
    }

    #[test]
    #[cfg(all(feature = "encoding", feature = "decoding"))]
    fn test_create_store_when_exists_should_fail() {
        let store = create_test_store("create_store_when_exists_should_fail");
        // Try to create the store again, should error
        let result = create_store(&store);
        assert!(result.is_err());
    }

    #[test]
    #[cfg(feature = "encoding")]
    fn test_create_repo_creates_directories() {
        let repo = temp_dir().join("lcas_testing_repo_dirs");
        let _ = remove_dir_all(&repo);

        create_repo(&repo).unwrap();

        assert!(repo.join("chunks").exists());
        assert!(repo.join("manifests").exists());
    }

    #[test]
    #[cfg(feature = "encoding")]
    fn test_create_repo_on_file() {
        let repo = temp_dir().join("lcas_testing_repo_file");
        let _ = remove_dir_all(&repo);

        fs::write(&repo, "Testing data.").unwrap();

        let err = create_repo(&repo).unwrap_err();
        assert!(err.to_string().contains("Already exists!"));
    }

    #[test]
    #[cfg(feature = "encoding")]
    fn test_create_repo_on_dir() {
        let repo = temp_dir().join("lcas_testing_repo_dir");
        let _ = remove_dir_all(&repo);

        fs::create_dir(&repo).unwrap();

        let err = create_repo(&repo).unwrap_err();
        assert!(err.to_string().contains("Already exists!"));
    }

    #[test]
    #[cfg(feature = "decoding")]
    fn test_create_temp_file_overflow_test() {
        let dir = temp_dir().join("lcas_temp_file_overflow_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        for _i in 0..1000 {
            let file_name = super::get_temp_file(None, &dir);
            File::create_new(&file_name).unwrap();
            assert!(file_name.exists());
        }
    }

    #[test]
    #[cfg(feature = "decoding")]
    fn test_get_temp_file_returns_unique_names() {
        let dir = temp_dir().join("lcas_temp_file_unique_test");
        let _ = std::fs::create_dir_all(&dir);
        let file1 = super::get_temp_file(None, &dir);
        std::fs::File::create(dir.join(&file1)).unwrap();
        let file2 = super::get_temp_file(None, &dir);
        assert_ne!(file1, file2);
        let _ = remove_dir_all(&dir);
    }

    #[test]
    #[cfg(feature = "decoding")]
    fn test_make_chunk_executable_sets_permissions() {
        let dir = temp_dir().join("lcas_executable_test");
        let _ = fs::create_dir_all(dir.join("chunks"));
        let chunk_hash = "testchunk".to_string();
        let chunk_path = dir.join("chunks").join(&chunk_hash);
        File::create(&chunk_path).unwrap();

        super::make_chunk_executable(&chunk_hash, &dir).unwrap();

        let perms = fs::metadata(&chunk_path).unwrap().permissions();
        assert_eq!(perms.mode() & 0o111, 0o111);

        let _ = remove_dir_all(&dir);
    }

    #[test]
    #[cfg(all(feature = "encoding", feature = "decoding"))]
    fn test_create_and_load_artifact() {
        use std::path::PathBuf;

        use crate::{build, install_artifact};

        let store = create_test_store("artifact");
        let input_dir = temp_dir().join("lcas_artifact_test");

        let _ = fs::remove_dir_all(&input_dir);
        fs::create_dir_all(&input_dir).unwrap();
        fs::write(input_dir.join("file1.txt"), b"Hello, world!").unwrap();
        fs::write(input_dir.join("file2.txt"), b"Another file.").unwrap();

        // Create a nested directory and files inside it
        let nested_dir = input_dir.join("nested");
        fs::create_dir(&nested_dir).unwrap();
        fs::write(nested_dir.join("nested1.txt"), b"Nested file 1.").unwrap();
        fs::write(nested_dir.join("nested2.txt"), b"Nested file 2.").unwrap();

        // Create a deeper nested directory
        let deeper_nested_dir = nested_dir.join("deeper");
        fs::create_dir(&deeper_nested_dir).unwrap();
        fs::write(deeper_nested_dir.join("deepfile.txt"), b"Deep nested file.").unwrap();

        build(
            &input_dir,
            &PathBuf::from(&store.repos.first().unwrap()),
            "test_artifact",
        )
        .unwrap();

        install_artifact(&"test_artifact".to_string(), &store).unwrap();
    }

    #[test]
    #[cfg(all(feature = "encoding", feature = "decoding"))]
    fn test_create_and_load_artifact_multirepo() {
        use std::path::PathBuf;

        use crate::{build, install_artifact};

        let store_a = create_test_store("multirepo_a");
        let store_b = create_test_store("multirepo_b");

        let store = Store {
            kind: store_a.kind,
            repos: vec![
                store_a.repos.first().unwrap().clone(),
                store_b.repos.first().unwrap().clone(),
            ],
            cache_path: store_a.cache_path,
            path: store_a.path,
        };

        let input_dir = temp_dir().join("lcas_artifact_test_multirepo");

        // First artifact
        let _ = fs::remove_dir_all(&input_dir);
        fs::create_dir_all(&input_dir).unwrap();
        fs::write(input_dir.join("file1.txt"), b"Hello, world!").unwrap();

        build(
            &input_dir,
            &PathBuf::from(&store.repos.first().unwrap()),
            "test_artifact_a",
        )
        .unwrap();

        // Second artifact
        let _ = fs::remove_dir_all(&input_dir);
        fs::create_dir_all(&input_dir).unwrap();
        fs::write(input_dir.join("file1.txt"), b"Hello, world!").unwrap();

        build(
            &input_dir,
            &PathBuf::from(&store.repos.first().unwrap()),
            "test_artifact_b",
        )
        .unwrap();

        install_artifact(&"test_artifact_a".to_string(), &store).unwrap();
        install_artifact(&"test_artifact_b".to_string(), &store).unwrap();
    }
}
