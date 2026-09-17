import path from 'node:path';

/** @type {import('@rspack/core').Configuration} */
export default {
  entry: './index',
  output: {
    filename: '[id].xxxx.js',
    chunkFilename: '[id].xxxx.js',
  },
  stats: {
    assets: true,
    modules: true,
  },
};
