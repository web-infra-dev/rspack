/**
 * @type {import('@rspack/cli').Configuration}
 */
export default {
  entry: './index.js',
  output: {
    module: true,
    chunkFormat: 'module',
    library: {
      type: 'module',
    },
  },
};
