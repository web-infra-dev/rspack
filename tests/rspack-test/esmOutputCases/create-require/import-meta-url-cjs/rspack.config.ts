import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    filename: 'main.js',
    library: {
      type: 'commonjs2',
    },
    module: false,
  },
  optimization: {
    runtimeChunk: false,
  },
  module: {
    parser: {
      javascript: {
        createRequire: true,
        importMeta: false,
      },
    },
  },
});
