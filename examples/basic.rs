#[cfg(all(feature = "encoding", feature = "decoding"))]
fn main() -> Result<(), String> {
    use std::path::absolute;
    use std::{fs, path::Path};

    use lcas::{build, create_repo, create_store, install_artifact};

    // Helper variables
    // `input_dir` is the artifact, likely produced by a build system etc. This is what we want to "transmit".
    let input_dir = absolute(Path::new("./example_dir")).unwrap();
    // `repo_dir` is the Repo's contained directory, and should be hosted on a web server, as a directory, etc.
    let repo_dir = absolute(Path::new("./example_repo")).unwrap();
    // `store_dir` is the local path for the local store, and will be where the Store is placed, and each artifact inside.
    let store_dir = absolute(Path::new("./example_store")).unwrap();

    // Create an example repo and a store, *locally*
    create_repo(repo_dir.as_path())?;
    create_store(&store_dir)?;

    // Create an example artifact
    fs::create_dir_all(Path::new("./example_dir/nested_dir/super_nested_dir")).unwrap();
    fs::write("./example_dir/a", "Wow a file").unwrap();
    fs::write("./example_dir/nested_dir/b", "Wow another file, shocking.").unwrap();
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
    )
    .expect("Build Failure");

    // Install the resulting manifest into an artifact in the `store_dir`
    install_artifact(&"generic".to_string(), store_dir.as_path(), &repo_dir);

    Ok(())
}

#[cfg(not(all(feature = "encoding", feature = "decoding")))]
fn main() {
    use core::panic;

    panic!("You need the encoding and decoding features to run this example");
}
