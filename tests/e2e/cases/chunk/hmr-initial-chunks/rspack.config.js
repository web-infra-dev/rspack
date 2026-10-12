import fs from 'node:fs';
import { rspack } from '@rspack/core';

const mode = JSON.parse(
  fs.readFileSync(new URL('./mode.json', import.meta.url), 'utf8'),
);
const nativeCss = mode === 'native';
const esm = mode === 'esm';

export default {
  mode: 'development',
  entry: {
    app: './src/app.js',
    trigger: './src/trigger.js',
    unrelated: './src/unrelated.js',
  },
  experiments: { outputModule: esm },
  output: {
    ...(esm && { module: true, chunkFormat: 'module', chunkLoading: 'import' }),
    filename: esm ? 'initial/[name].js' : 'initial/[name].[fullhash].js',
    chunkFilename: 'async/[name].[contenthash].js',
    cssFilename: 'styles/[name].[contenthash].css',
    cssChunkFilename: 'async/[name].[contenthash].css',
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
      nativeCss
        ? { test: /\.css$/, type: 'css' }
        : {
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
          scriptLoading: esm ? 'module' : 'defer',
          template: './src/index.html',
        }),
    ),
    ...(nativeCss
      ? []
      : [
          new rspack.CssExtractRspackPlugin({
            filename: 'styles/[name].[contenthash].css',
            chunkFilename: 'async/[name].[contenthash].css',
          }),
        ]),
  ],
  devServer: { hot: true, liveReload: false },
};
