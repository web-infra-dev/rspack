## Development scripts

This directory contains scripts for developing, benchmarking, and releasing Rspack.

## Usage

Run JavaScript scripts from the repository root with Node.js:

```bash
node ./scripts/x.mjs --help
```

The development CLI is also available through `./x` or `pnpm x`. See the [development commands](../.agents/DEVELOPMENT.md) for common build and test workflows.

Declare script dependencies in [package.json](./package.json).

## Writing scripts with zx

When using [zx](https://github.com/google/zx), import its globals explicitly so the script can run with Node.js:

```js
import 'zx/globals';
```
