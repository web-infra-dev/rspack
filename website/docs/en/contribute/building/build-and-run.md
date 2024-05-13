# How to build and run the compiler

Please see [prerequisites](./prerequisites.md) for setting up Rust and Node.js.

## Install Node.js dependencies

Install Node.js dependencies via [pnpm](https://pnpm.io/).

```bash
# enable pnpm with corepack, only available on node >= `v14.19.0`
corepack enable

# or install pnpm directly
npm install -g pnpm@8

# Install dependencies
pnpm install
```

## Building Rspack

- Run `cargo build` to compile Rust code.
- Run `pnpm run build:cli:debug` to compile both Node.js and Rust code.

The built binary is located at `packages/rspack-cli/bin/rspack`.
