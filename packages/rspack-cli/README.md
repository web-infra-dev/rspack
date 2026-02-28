<picture>
  <img alt="Rspack Banner" src="https://assets.rspack.rs/rspack/rspack-banner.png">
</picture>

# @rspack/cli

Command-line interface for rspack.

## Installation

```bash
pnpm add -D @rspack/cli
# or
npm install -D @rspack/cli
# or
yarn add -D @rspack/cli
```

## Required dependencies

The `rspack dev` and `rspack preview` commands require [@rspack/dev-server](https://www.npmjs.com/package/@rspack/dev-server) to be installed as a peer dependency:

```bash
pnpm add -D @rspack/dev-server
# or
npm install -D @rspack/dev-server
# or
yarn add -D @rspack/dev-server
```

If you try to use these commands without installing `@rspack/dev-server`, you will see a helpful error message with installation instructions.

## Wasm test

See [@rspack/test-tools](../rspack-test-tools) for details.

## Documentation

See [https://rspack.rs](https://rspack.rs) for details.

## License

Rspack is [MIT licensed](https://github.com/web-infra-dev/rspack/blob/main/LICENSE).
