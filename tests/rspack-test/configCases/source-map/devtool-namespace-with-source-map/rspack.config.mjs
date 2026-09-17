import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
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
  devtool: 'source-map',
};
