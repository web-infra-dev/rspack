const { rspack } = require('@rspack/core');
const { ReactRefreshRspackPlugin } = require('@rspack/plugin-react-refresh');

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  entry: './src/index.jsx',
  mode: 'development',
  devtool: false,
  stats: 'errors-warnings',
  resolve: {
    extensions: ['...', '.jsx'],
  },
  module: {
    rules: [
      {
        test: /\.jsx$/,
        loader: 'builtin:swc-loader',
        options: {
          detectSyntax: 'auto',
          jsc: {
            transform: {
              react: {
                runtime: 'automatic',
                development: true,
                refresh: true,
              },
            },
          },
        },
      },
    ],
  },
  lazyCompilation: {
    entries: false,
    imports: true,
  },
  devServer: {
    hot: true,
  },
  plugins: [
    new rspack.HtmlRspackPlugin({ template: './src/index.html' }),
    new ReactRefreshRspackPlugin(),
    {
      apply(compiler) {
        compiler.__buildCount = 0;
        compiler.hooks.done.tap('BuildCountPlugin', () => {
          compiler.__buildCount++;
        });
      },
    },
  ],
};
