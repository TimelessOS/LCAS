use std::{fs, path::Path};

#[cfg(feature = "encoding")]
#[test]
fn main() {
    use lcas::{build, create_repo};

    let input_dir = Path::new("./example_dir");
    let repo_dir = Path::new("./example_repo");

    if !repo_dir.exists() {
        create_repo(repo_dir).unwrap();
    }

    fs::create_dir_all(Path::new("./example_dir/nested_dir/super_nested_dir")).unwrap();
    fs::write("./example_dir/a", "Wow a file").unwrap();
    fs::write("./example_dir/nested_dir/b", "Wow another NOW NESTED file").unwrap();
    fs::write(
        "./example_dir/nested_dir/super_nested_dir/c",
        "Wow very complex. This works well!",
    )
    .unwrap();

    build(input_dir, repo_dir, &"generic".to_string()).expect("Build Failure");
}
