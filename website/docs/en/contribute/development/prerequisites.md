# Prerequisites

Rspack is built using [Rust](https://rust-lang.org/) and [NAPI-RS](https://napi.rs/), then released as [Node.js](https://nodejs.org/) packages.

## Setup Rust

- Install Rust using [rustup](https://rustup.rs/).
- If you are using VS Code, we recommend installing the [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) extension.

## Setup Node.js

### Install Node.js

We recommend using the LTS version of Node.js 20.

Check the current Node.js version with the following command:

```bash
node -v
```

If you do not have Node.js installed in your current environment, you can use [nvm](https://github.com/nvm-sh/nvm) or [fnm](https://github.com/Schniz/fnm) to install it.

Here is an example of how to install via nvm:

```bash
# Install Node.js LTS
nvm install 20 --lts

# Switch to Node.js LTS
nvm use 20
```
