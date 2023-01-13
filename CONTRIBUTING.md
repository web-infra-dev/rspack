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

## Release

### Prerequisite

1. Making sure you have permission to access organization `@rspack` in npmjs.com
2. `Zig` compiler, you could install it by running `brew install zig` on Macos, for other OS, please refer https://ziglang.org/learn/getting-started/#installing-zig
3. Installing `Linux` target toolchain (for now, only two target Macos and linux are fairly enough), install `linux` target with command `rustup target add x86_64-unknown-linux-gnu`, if you are using Macos with arm architecture, you also need to run `rustup target add x86_64-apple-darwin`.

### Step

1. Making sure you have logged into npm
2. Building packages `./x build cli:release`.
3. `pnpm changeset`
4. `pnpm bump` (for stable release) or `pnpm version:snapshot` (for snapshot release)
5. `pnpm release`

## Testing

We currently have two sets of test suits, one for rust and one for node.

### Rust Testing

- `cargo test` will run all the rust side tests, which includes standalone tests for core functionality and plugins.
- `UPDATE=1 cargo test` will automatically update the failed snapshot

### Node Testing

```sh
# you need to build js package before running tests
pnpm run build && pnpm run test:js
```

### Node Testing Suit Overview

We use jest for nodejs tests, The most important test cases are the case in the packages/rspack. most of these cases comes from webpack https://github.com/webpack/webpack/tree/main/test because we want to make sure that rspack can work as same as webpack.

There are three kinds of integration cases in rspack/core.

#### case.test.ts

Cases are used to test normal build behavior, we use these cases to test against bundler core functionality, like `entry`, `output`, `module` `resolve`, etc. it will first build your test file to test whether the input could be compiled successfully, then it will use the bundled test file to run test cases in the test file to test bundler's all kinds of behavior.

#### configCase.test.ts

Cases are used to test custom build behavior, you could use custom `webpack.config.js` to override default build behavior, you can use these cases to test against behavior related to specific config.

##### statsTestCase.test.ts

Cases are used to test your stats, By Default we will use jest's snapshot to snapshot your stats, and we **highly** recommend to **avoid** snapshot except statsCase. you can use statsCase to test behaviors like code splitting | bundle splitting, which is hard to test by just running code.

### Useful Scripts

We have written some useful scripts in `scripts` folder to help with some tedious things.

NOTE: We should upload `node_modules` to git while developing scripts.

##### update_swc_version.js

This script will scan all of `cargo.toml` and replace dependency version with the corresponding version in https://github.com/swc-project/swc/tree/main

This script can config with the following

- `swc_version` - the swc version tag, we need update it when upgrade swc.
- `swc_packages` - the regex to match dependency which is in swc repo

WARNING: We have hacked `@iarna/toml/stringify.js` to make the output match our format while developing this script

##### check_rust_dependency.js

This script will print the duplicate dependencies in `cargo.toml` for all crates.

# Debugging with VSCode

1. Install `go install github.com/go-delve/delve/cmd/dlv@latest`
2. Install vscode extension [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) and [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb)
3. build rspack-cli and napi binding by run `./x build cli:debug`
4. In Vscode's `Run and Debug` tab, select `debug-rspack` to start debugging the initial launch of rspack-cli, This task is configured in `.vscode/launch.json`, which launch node debugger and rust debugger together. so you can debug both rust and nodejs code.
