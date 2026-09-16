/** @type {import("../../../../").Configuration} */
export default {
  output: {
    assetModuleFilename: '[path][name][ext]',
  },
  module: {
    parser: {
      javascript: {
        // this is always true in rspack
        // dynamicUrl: true
      },
    },
  },
};
