# rspack_binding_build

Build script for rspack_binding_builder.

## Usage

In `build.rs`, you can use `rspack_binding_build::setup` to setup the build script.

> **Note:** The following code block uses `ignore` because it is intended to be placed in a `build.rs` file, not in regular Rust source code.

```rust,ignore
fn main() {
  rspack_binding_build::setup();
}
```

## Guide

[Rspack Custom binding](https://rspack-contrib.github.io/rspack-rust-book/custom-binding/getting-started/index.html)
