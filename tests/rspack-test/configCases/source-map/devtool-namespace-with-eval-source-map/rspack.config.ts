import { defineConfig } from '@rspack/cli';

import path from 'node:path';

export default defineConfig({
  mode: 'development',
  entry: {
    'entry-a': [path.join(import.meta.dirname, './src/entry-a')],
    'entry-b': [path.join(import.meta.dirname, './src/entry-b')],
  },

  output: {
    filename: '[name]-bundle.js',
    library: { type: 'commonjs', name: 'library-[name]' },
    devtoolNamespace: 'library-[name]',
  },
  devtool: 'eval-source-map',
});
