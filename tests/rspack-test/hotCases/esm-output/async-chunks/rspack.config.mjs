/** @type {import("../../../../types").Configuration} */
export default {
  mode: 'development',
  output: {
    module: true,
    chunkFormat: 'module',
    filename: '[name].mjs',
    chunkFilename: '[name].chunk.mjs',
    enabledLibraryTypes: ['module'],
  },
  optimization: {
    minimize: false,
  },
};
