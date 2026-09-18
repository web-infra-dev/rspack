/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    bundle0: {
      import: './index.js',
      baseUri: 'my-scheme://baseuri',
      publicPath: '/',
    },
  },
  output: {
    assetModuleFilename: '[name][ext]',
  },
  target: 'web',
};
