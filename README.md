# Linked Content Addressed Storage

[![codecov](https://codecov.io/gh/TimelessOS/LCAS/graph/badge.svg?token=5W9K4IK81O)](https://codecov.io/gh/TimelessOS/LCAS)

A simple yet complex method of storing large directory structures with duplicate files

## Examples

```rust
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
    cache_path,
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
build(&input_dir, &repo_dir, "generic")?;

// Install the resulting manifest into an artifact in the `store_dir`
install_artifact(&"generic".to_string(), &store)?;

Ok(())
```

For further examples please check [the examples in the source tree.](https://github.com/TimelessOS/LCAS/tree/main/examples)

## Terminology

- Repo: The storage location of all uploaded chunks, artifacts, and manifests. Commonly used by the distributer of directories.
- Store: The storage location of all downloaded chunks and manifests, alongside the built artifacts. Commonly used by the downloader of directories.
- Manifest: A list of every file's relation to a chunk used to recreate the Artifact.
- Artifact: The actual target directory.
- Chunk: A raw deduplicated file.

Please note: There is minor differences between implementation depending on whether they are in relation to the Store or Repo.
