import { defineConfig } from '@rspack/cli';

import path from 'node:path';

export default defineConfig({
  entry: './src/index.js',
  externals: {
    'node:events': 'module node:events',
  },
  output: {
    library: {
      type: 'modern-module',
      preserveModules: path.resolve(import.meta.dirname, 'src'),
    },
  },
  optimization: {
    mangleExports: 'size',
    minimize: false,
  },
});
