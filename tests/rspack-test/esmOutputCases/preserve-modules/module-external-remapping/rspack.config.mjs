import path from 'node:path';

/** @type {import('@rspack/core').Configuration} */
export default {
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
};
