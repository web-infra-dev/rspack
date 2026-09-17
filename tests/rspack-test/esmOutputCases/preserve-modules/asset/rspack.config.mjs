import path from 'node:path';

/**@type {import('@rspack/core').Configuration} */
export default {
  target: 'web',
  entry: './src/index.js',
  module: {
    rules: [
      {
        test: /\.png$/,
        type: 'asset/resource',
      },
    ],
  },
  output: {
    library: {
      type: 'modern-module',
      preserveModules: path.resolve(import.meta.dirname, 'src'),
    },
  },
};
