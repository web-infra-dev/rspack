import { defineConfig } from '@rspack/cli';

export default [false, true].map((minimize, index) =>
  defineConfig({
    mode: 'production',
    entry: {
      main: './index.js',
      jsonParse: './json-parse.js',
      array: './array.js',
      javascript: './javascript.js',
      conditional: './conditional.js',
      loop: './loop.js',
    },
    output: {
      filename: `[name]${index}.mjs`,
      module: true,
      library: { type: 'modern-module' },
    },
    optimization: { minimize, runtimeChunk: false },
  }),
);
