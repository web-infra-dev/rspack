const {
  CssExtractRspackPlugin,
  SubresourceIntegrityPlugin,
} = require('@rspack/core');
const fs = require('fs');
const path = require('path');

/** @type {import("@rspack/core").Configuration} */
module.exports = (_, { testPath }) => ({
  target: 'web',
  output: {
    crossOriginLoading: 'anonymous',
  },
  experiments: {
    css: false,
  },
  optimization: {
    chunkIds: 'named',
    moduleIds: 'named',
    minimize: false,
    // Move the extracted css of every async chunk into a chunk of its own, so
    // that chunk holds `css/mini-extract` modules and nothing else.
    splitChunks: {
      chunks: 'async',
      cacheGroups: {
        styles: {
          type: 'css/mini-extract',
          name: 'styles',
          chunks: 'async',
          enforce: true,
        },
      },
    },
  },
  plugins: [
    new CssExtractRspackPlugin(),
    new SubresourceIntegrityPlugin(),
    {
      apply(compiler) {
        compiler.hooks.afterEmit.tap('AfterEmitPlugin', () => {
          const content = fs.readFileSync(
            path.resolve(testPath, 'bundle0.js'),
            'utf-8',
          );
          expect(content).toContain('sriExtractCssHashes');
          expect(content).toContain('"styles": "sha384-');
        });
      },
    },
  ],
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'javascript/auto',
        use: [CssExtractRspackPlugin.loader, 'css-loader'],
      },
    ],
  },
});
