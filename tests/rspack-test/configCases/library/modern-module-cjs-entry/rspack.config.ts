import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    main: { import: './index.js', filename: 'bundle.mjs' },
  },
  output: {
    module: true,
    library: {
      type: 'module',
    },
  },
  module: {
    parser: {
      javascript: {
        importMeta: false,
      },
    },
  },
  optimization: {
    concatenateModules: true,
  },
});
