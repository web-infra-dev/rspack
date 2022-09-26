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

## Testing

We currently have two sets of test suits, one for rust and one for node.

### Rust Testing

- cargo test will run all the rust side tests, which includes standalone tests for core functionality and plugins.

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

we have written some useful scripts in `scripts` folder to help with some tedious things.

NOTE: we should upload `node_modules` to git while developing scripts.

##### update_swc_version.js

this script will scan all of `cargo.toml` and replace dependency version with the corresponding version in https://github.com/swc-project/swc/tree/main

this script can config with the following

* swc_version - the swc version tag, we need update it when upgrade swc.
* swc_packages - the regex to match dependency which is in swc repo

WARNING: we have hacked `@iarna/toml/stringify.js` to make the output match our format while developing this script
