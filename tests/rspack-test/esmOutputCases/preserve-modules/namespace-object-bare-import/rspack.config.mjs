import path from 'node:path';

/** @type {import('@rspack/core').Configuration} */
export default {
  entry: './src/index.js',
  output: {
    library: {
      type: 'modern-module',
      preserveModules: path.resolve(import.meta.dirname, 'src'),
    },
  },
};
