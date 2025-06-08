# Linked Content Addressed Storage

For projects that use large amounts of duplicate files

## Using

### Terminology

- Repo: The storage location of all uploaded chunks, artifacts, and manifests. Commonly used by the distributer of directories.
- Store: The storage location of all downloaded chunks and manifests, alongside the built artifacts. Commonly used by the downloader of directories.
- Manifest: A list of every file's relation to a chunk used to recreate the Artifact.
- Artifact: The actual target directory.
- Chunk: A raw deduplicated file.

Please note: There is minor differences between implementation depending on whether they are in relation to the Store or Repo.

### Examples

For further examples please check [the examples in the source tree.](https://github.com/TimelessOS/LCAS/tree/main/examples)

## Contributing

### TODO

- [ ] Networking support for repos
- [ ] Error handling (Should be propogated upwards)
- [ ] Proper tests
- [ ] Proper/Better documentation
- [ ] Windows Support (UNIX-Like only)
