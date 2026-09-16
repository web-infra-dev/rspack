/**
 * @type {import('@rspack/core').RspackOptions}
 */
export default {
  experiments: {
    outputModule: true,
  },
  output: {
    module: true,
    chunkFormat: 'module',
  },
};
