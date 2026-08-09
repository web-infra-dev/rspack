const { rspack } = require('@rspack/core');

/** @type { import('@rspack/core').RspackOptions } */
module.exports = {
  context: __dirname,
  entry: {
    main: './src/index.js',
  },
  mode: 'development',
  stats: 'none',
  devtool: false,
  plugins: [new rspack.HtmlRspackPlugin()],
  lazyCompilation: {
    entries: false,
    imports: true,
  },
  devServer: {
    hot: true,
  },
};
