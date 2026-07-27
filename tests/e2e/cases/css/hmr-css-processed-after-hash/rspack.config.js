const { rspack } = require('@rspack/core');

/** @type { import('@rspack/core').RspackOptions } */
module.exports = {
  context: __dirname,
  mode: 'development',
  entry: {
    main: './src/index.js',
  },
  plugins: [
    new rspack.HtmlRspackPlugin({
      template: './src/index.html',
      inject: 'body',
    }),
    new rspack.CssExtractRspackPlugin(),
    // rewrites the stylesheet during processAssets with content that changes
    // whenever the compilation hash does, without touching any css module
    new rspack.BannerPlugin({
      raw: true,
      test: /\.css$/,
      banner: ({ hash }) => `#root::after { content: "${hash}"; }\n`,
    }),
  ],
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'javascript/auto',
        use: [rspack.CssExtractRspackPlugin.loader, 'css-loader'],
      },
    ],
  },
};
