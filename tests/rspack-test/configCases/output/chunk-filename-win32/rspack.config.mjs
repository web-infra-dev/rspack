import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index.js',
  output: {
    chunkFilename: path.win32.join('./', 'js/[name].[chunkhash:8].chunk.js'),
  },
};
