import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    main: { import: './index.js', filename: 'bundle.mjs' },
    json: { import: './index.json', filename: 'json.mjs' },
  },
  output: {
    module: true,
    library: {
      type: 'modern-module',
    },
  },
  module: {
    parser: {
      javascript: {
        importMeta: false,
      },
    },
  },
});
