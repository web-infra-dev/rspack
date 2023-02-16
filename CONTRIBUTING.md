# Rspack Contributing Guide

Thank you for your interest in contributing to Rspack! Before starting your contribution, please take a moment to read the following guidelines.

## Sending a Pull Request

1. [Fork](https://help.github.com/articles/fork-a-repo/) the Rspack repository into your own GitHub account.
2. [Clone](https://help.github.com/articles/cloning-a-repository/) the repository to your local.
3. Checkout a new branch from `main`.
4. Set up the development environment, you can read the "Setup Development Environment" section below to learn about it.
5. If you've fixed a bug or added code that should be tested, then add some tests.
6. Make sure all the tests pass, you can read the "Testing" section below to learn about it.
7. Run `pnpm run lint:js` and `pnpm run lint:rs` to check the code style.
8. If you've changed some Node.js packages, you should add a new [changeset](https://github.com/changesets/changesets). Run `pnpm run changeset`, select the changed packages and add the changeset info.
9. Submit the Pull Request, make sure all CI runs pass.
10. The maintainers will review your Pull Request soon.

When submitting a Pull Request, please note the following:

- Keep your PRs small enough, so that each PR only addresses a single issue or adds a single feature.
- Please include an appropriate description in the PR, and link related issues.

## Setup Development Environment

### Setup Rust

- Install Rust using [rustup](https://rustup.rs/).
- If you are using VSCode, we recommend installing the [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) extension.

### Setup Node.js

#### Install Node.js

We recommend using the LTS version of Node.js 16. You can check your currently used Node.js version with the following command:

```bash
node -v
#v16.18.0
```

If you do not have Node.js installed in your current environment, you can use [nvm](https://github.com/nvm-sh/nvm) or [fnm](https://github.com/Schniz/fnm) to install it.

Here is an example of how to install the Node.js 16 LTS version via nvm:

```bash
# Install the LTS version of Node.js 16
nvm install 16 --lts

# Make the newly installed Node.js 16 as the default version
nvm alias default 16

# Switch to the newly installed Node.js 16
nvm use 16
```

#### Install Dependencies

Install Node.js dependencies via [pnpm](https://pnpm.io/).

```bash
# enable pnpm with corepack
corepack enable

# or install pnpm directly
npm install -g pnpm@7

# Install dependencies
pnpm run init
```

### Setup Other Dependencies

- Install [protoc](https://grpc.io/docs/protoc-installation/) for building `sass-embedded`.

### Final

- Open Rspack project.
- Run `cargo build` to see that is everthing ok.

## Testing

We currently have two sets of test suits, one for Rust and one for Node.js.

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

## Release

### Prerequisite

1. Making sure you have permission to access organization `@rspack` in npmjs.com
2. `Zig` compiler, you could install it by running `brew install zig` on macOS, for other OS, please refer https://ziglang.org/learn/getting-started/#installing-zig
3. Installing `Linux` target toolchain (for now, only two target macOS and Linux are fairly enough), install `linux` target with command `rustup target add x86_64-unknown-linux-gnu`, if you are using macOS with arm architecture, you also need to run `rustup target add x86_64-apple-darwin`.

### Step

1. Making sure you have logged into npm
2. Building packages `./x build cli:release`.
3. `pnpm changeset`
4. `pnpm bump` (for stable release) or `pnpm version:snapshot` (for snapshot release)
5. `pnpm release`

# Debugging with VSCode

1. Install `go install github.com/go-delve/delve/cmd/dlv@latest`
2. Install vscode extension [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) and [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb)
3. build rspack-cli and napi binding by run `./x build cli:debug`
4. In Vscode's `Run and Debug` tab, select `debug-rspack` to start debugging the initial launch of rspack-cli, This task is configured in `.vscode/launch.json`, which launch node debugger and rust debugger together. so you can debug both rust and nodejs code.
