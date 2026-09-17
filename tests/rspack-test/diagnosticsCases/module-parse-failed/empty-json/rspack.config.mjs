/**
 * @type {import('@rspack/core').RspackOptions}
 */
export default {
  context: import.meta.dirname,
  devtool: false,
  optimization: {
    concatenateModules: true,
    minimize: false,
  },
};
