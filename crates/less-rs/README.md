# less-rs

A Rust port of the Less CSS compiler, imported from
[`less/less-rs`](https://github.com/less/less-rs) at commit
`9dd5e79e`. The imported source retains its Apache-2.0 license.

```rust
let output = less_rs::compile("@color: red; a { color: @color }", &less_rs::Options::default())?;
assert_eq!(output.css, "a {\n  color: red;\n}\n");
```

This crate provides the compiler library as an Rspack Cargo workspace member.
It targets less.js 4.9.1 semantics and supports variables, mixins, imports,
custom Rust functions, and source maps. JavaScript evaluation and JavaScript
plugins are unsupported.

Default features are `data-uri` and `sourcemap`. Optional features include
`json`, `serde`, `miette`, `http`, and `arena-stats`.

The CLI, Node.js and WebAssembly bindings, benchmarks, and upstream test
fixtures remain in the upstream project.
