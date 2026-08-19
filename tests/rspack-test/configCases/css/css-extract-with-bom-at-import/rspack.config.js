const { rspack } = require('@rspack/core');

// `import: false` keeps the `@import` inside the module content instead of
// splitting it into its own module, which is how dart-sass output reaches the
// plugin: a BOM, immediately followed by `@import url(...)`.
const cssLoader = { loader: 'css-loader', options: { import: false } };

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  target: 'web',
  node: false,
  entry: {
    main: './index.js',
  },
  output: {
    filename: '[name].js',
  },
  module: {
    rules: [
      {
        test: /first\.css$/,
        type: 'javascript/auto',
        use: [rspack.CssExtractRspackPlugin.loader, cssLoader],
      },
      {
        test: /imported\.css$/,
        type: 'javascript/auto',
        use: [
          rspack.CssExtractRspackPlugin.loader,
          cssLoader,
          require.resolve('./bom-loader.js'),
        ],
      },
    ],
  },
  plugins: [
    new rspack.CssExtractRspackPlugin({
      filename: '[name].css',
    }),
  ],
  experiments: {
    css: false,
  },
};
