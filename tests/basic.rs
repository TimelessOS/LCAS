use std::{fs, path::Path};

#[cfg(all(feature = "encoding", feature = "decoding"))]
#[test]
fn main() {
    use std::path::absolute;

    use lcas::{build, create_repo, create_store, install_artifact};

    let input_dir = absolute(Path::new("./example_dir")).unwrap();
    let repo_dir = absolute(Path::new("./example_repo")).unwrap();
    let store_dir = absolute(Path::new("./example_store")).unwrap();

    if !repo_dir.exists() {
        create_repo(repo_dir.as_path()).unwrap();
    }
    if !store_dir.exists() {
        create_store(&store_dir).unwrap();
    }
    fs::create_dir_all(Path::new("./example_dir/nested_dir/super_nested_dir")).unwrap();
    fs::write("./example_dir/a", "Wow a file").unwrap();
    fs::write("./example_dir/nested_dir/b", "Wow another NOW NESTED file").unwrap();
    fs::write(
        "./example_dir/nested_dir/super_nested_dir/c",
        "Wow very complex. This works well!",
    )
    .unwrap();

    build(
        input_dir.as_path(),
        repo_dir.as_path(),
        &"generic".to_string(),
    )
    .expect("Build Failure");

    install_artifact(&"generic".to_string(), store_dir.as_path(), &repo_dir);
}
