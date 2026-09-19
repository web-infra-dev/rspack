import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';

const {
  experiments: { VirtualModulesPlugin },
} = rspack;

export default defineConfig({
  entry: {
    main: './index.js',
  },
  output: {
    filename: '[name].js',
  },
  plugins: [
    new VirtualModulesPlugin({
      'index.js':
        'const a = require("a").default; const b = require("b").default; export default a + b;',
      'node_modules/a.js': 'export default 1;',
      'node_modules/b.js': 'export default 2;',
    }),
  ],
});
