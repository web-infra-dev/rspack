const { rspack } = require('@rspack/core');

/** @type { import('@rspack/core').RspackOptions } */
module.exports = {
  context: __dirname,
  entry: {
    main: './src/index.js',
  },
  stats: 'none',
  mode: 'development',
  plugins: [new rspack.HtmlRspackPlugin()],
  lazyCompilation: {
    entries: true,
    imports: true,
    // Set custom prefix for lazy compilation
    prefix: '/custom-lazy-endpoint-',
  },
  devServer: {
    hot: true,
  },
};
