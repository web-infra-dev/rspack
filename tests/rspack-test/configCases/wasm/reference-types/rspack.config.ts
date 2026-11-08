import { defineConfig } from '@rspack/cli';

/** @typedef {import("@rspack/core").Compiler} Compiler */

export default defineConfig({
  output: {
    webassemblyModuleFilename: '[id].[hash].wasm',
  },
  experiments: {
    asyncWebAssembly: true,
  },
});
