// eslint-disable-next-line node/no-unpublished-require
const { SharePlugin } = require('@rspack/core').sharing;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  devtool: false,
  plugins: [
    new SharePlugin({
      shared: {
        '@scope/pkg': {},
        '@scope/pkg/styles': {},
      },
    }),
  ],
};
