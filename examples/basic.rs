#![warn(clippy::pedantic)]

use anyhow::Result;
#[cfg(all(feature = "encoding", feature = "decoding"))]
fn main() -> Result<()> {
    use std::path::absolute;
    use std::{fs, path::Path};

    use lcas::{RepoType, Store, build, create_repo, create_store, install_artifact};

    // Helper variables
    // `input_dir` is the artifact, likely produced by a build system etc. This is what we want to "transmit".
    let input_dir = absolute(Path::new("./example_dir"))?;
    // `repo_dir` is the Repo's contained directory, and should be hosted on a web server, as a directory, etc.
    let repo_dir = absolute(Path::new("./example_repo"))?;
    // `store_dir` is the local path for the local store, and will be where the Store is placed, and each artifact inside.
    let store_dir = absolute(Path::new("./example_store"))?;
    // `store_dir` is the local path for the local store, and will be where the Store is placed, and each artifact inside.
    let cache_path = absolute(Path::new("./example_cache"))?;

    let store = Store {
        kind: RepoType::Local,
        cache_path: cache_path,
        repo_path: repo_dir.to_string_lossy().to_string(),
        path: store_dir,
    };

    // Create an example repo and a store, *locally*
    let _ = create_repo(repo_dir.as_path());
    let _ = create_store(&store);

    // Create an example artifact
    fs::create_dir_all(Path::new("./example_dir/nested_dir/super_nested_dir"))?;
    fs::write("./example_dir/a", "Wow a file")?;
    fs::write("./example_dir/nested_dir/b", "Wow another file, shocking.")?;
    fs::write(
        "./example_dir/nested_dir/super_nested_dir/c",
        "Nested nested nested file",
    )
    .unwrap();

    // Compile the artifact into a manifest and chunks and store it
    build(
        input_dir.as_path(),
        repo_dir.as_path(),
        &"generic".to_string(),
    )?;

    // Install the resulting manifest into an artifact in the `store_dir`
    install_artifact(&"generic".to_string(), &store)?;

    Ok(())
}

#[cfg(not(all(feature = "encoding", feature = "decoding")))]
fn main() -> Result<()> {
    use anyhow::bail;

    bail!("You need the encoding and decoding features to run this example");
}

#[test]
#[cfg(all(feature = "encoding", feature = "decoding"))]
fn test_example() -> Result<()> {
    use std::{fs::remove_dir_all, path::Path};

    let _ = remove_dir_all(Path::new("./example_dir"));
    let _ = remove_dir_all(Path::new("./example_store"));
    let _ = remove_dir_all(Path::new("./example_repo"));
    let _ = remove_dir_all(Path::new("./example_cache"));

    let val = main();

    let _ = remove_dir_all(Path::new("./example_dir"));
    let _ = remove_dir_all(Path::new("./example_store"));
    let _ = remove_dir_all(Path::new("./example_repo"));
    let _ = remove_dir_all(Path::new("./example_cache"));

    val
}
