# rspack_binding_build

Build script for rspack_binding_builder.

## Usage

In `build.rs`, you can use `rspack_binding_build::setup` to setup the build script.

```rust,ignore
fn main() {
  rspack_binding_build::setup();
}
```

## Guide

[Rspack Custom binding](https://rspack-contrib.github.io/rspack-rust-book/custom-binding/getting-started/index.html)