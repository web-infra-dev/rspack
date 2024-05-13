# Prerequisites

## Setup Rust

- Install Rust using [rustup](https://rustup.rs/).
- If you are using VSCode, we recommend installing the [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) extension.

## Setup Node.js

### Install Node.js

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
