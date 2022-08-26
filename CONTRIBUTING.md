# Set up development environment

## Setup rust

- Install rust using [rustup](https://rustup.rs/).
- Install [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) for VSCode.

## Setup node

- Install js dependencies using `pnpm install`.

## Setup other dependencies

- Install [protoc](https://grpc.io/docs/protoc-installation/) for building `sass-embedded`.

## Final

- Open rspack project.
- Run `cargo build` to see that is everthing ok.

<!-- # Testing

Run `cargo run gen_test_config_schema` to update the schema of `test.config.json`, if you change the `TestOptions` in `rspack_test`. -->
