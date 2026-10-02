import { rspack } from '@rspack/core';

export default {
  mode: 'development',
  entry: {
    app: './src/app.js',
    trigger: './src/trigger.js',
    unrelated: './src/unrelated.js',
  },
  experiments: { outputModule: true },
  output: {
    module: true,
    chunkFormat: 'module',
    chunkLoading: 'import',
    filename: 'initial/[name].js',
    chunkFilename: 'async/[name].[contenthash].js',
  },
  lazyCompilation: { entries: false, imports: true },
  optimization: {
    runtimeChunk: 'single',
    moduleIds: 'named',
    chunkIds: 'named',
    concatenateModules: false,
    splitChunks: {
      minSize: 0,
      chunks: 'all',
      cacheGroups: {
        default: false,
        defaultVendors: false,
        shared: { test: /shared\.(js|css)$/, name: 'shared', minChunks: 2 },
        added: { test: /added\.(js|css)$/, name: 'added', enforce: true },
      },
    },
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'javascript/auto',
        use: [rspack.CssExtractRspackPlugin.loader, 'css-loader'],
      },
    ],
  },
  plugins: [
    ...['app', 'trigger', 'unrelated'].map(
      (entry) =>
        new rspack.HtmlRspackPlugin({
          filename: entry === 'app' ? 'index.html' : `${entry}.html`,
          chunks: [entry],
          scriptLoading: 'module',
          template: './src/index.html',
        }),
    ),
    new rspack.CssExtractRspackPlugin({
      filename: 'styles/[name].[contenthash].css',
      chunkFilename: 'async/[name].[contenthash].css',
    }),
  ],
  devServer: { hot: true, liveReload: false },
};
