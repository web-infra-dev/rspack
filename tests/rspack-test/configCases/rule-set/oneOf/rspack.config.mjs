/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  output: {
    assetModuleFilename: '[name][ext]',
  },
  experiments: {
    css: false,
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        oneOf: [
          {
            use: ['./style-loader.mjs', './css-loader.mjs'],
            issuer: /\.(js)$/,
          },
          {
            // TODO: should not change source type when no pre/post loader
            // type: "asset/resource",
            issuer: /\.(css|scss|sass)$/,
          },
        ],
      },
    ],
  },
};
