const { rspack } = require('@rspack/core');

const cssLoader = { loader: 'css-loader', options: { sourceMap: true } };

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  target: 'web',
  node: false,
  devtool: 'source-map',
  entry: {
    // `main` imports plain.css, `bom` imports sm.css — identical content, but
    // sm.css additionally goes through a loader that prepends a BOM.
    main: './index.js',
    bom: './bom.js',
  },
  output: {
    filename: '[name].js',
  },
  module: {
    rules: [
      {
        test: /plain\.css$/,
        type: 'javascript/auto',
        use: [rspack.CssExtractRspackPlugin.loader, cssLoader],
      },
      {
        test: /sm\.css$/,
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
