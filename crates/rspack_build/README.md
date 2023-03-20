# rspack_build

This is only used for profiling

## Prerequisite

```sh
cargo install cargo-instruments  # if this not working try to run `brew install cargo-instruments`
```

## Profiling

```sh
cargo instruments --bin rspack_build --release --template alloc your_project_root_contains_test.config.json
```
