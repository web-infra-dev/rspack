import { rspack } from '@rspack/core';
import { fileURLToPath } from 'node:url';

const cssLoader = { loader: 'css-loader', options: { sourceMap: true } };

/** @type {import("@rspack/core").Configuration} */
export default {
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
          fileURLToPath(import.meta.resolve('./bom-loader.js')),
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
