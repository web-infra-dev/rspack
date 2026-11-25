import { HtmlRspackPlugin, SubresourceIntegrityPlugin } from '@rspack/core';

export default {
  target: 'web',
  entry: {
    shared: { import: './shared.js', runtime: 'runtime' },
    vendor: { import: './vendor.js', dependOn: 'shared' },
    app: { import: './app.js', dependOn: 'vendor' },
    other: { import: './other.js', dependOn: 'vendor' },
    sync: { import: './sync.js', dependOn: 'shared' },
  },
  output: {
    module: true,
    filename: '[name].mjs',
    chunkFilename: '[name].mjs',
    publicPath: '/',
    crossOriginLoading: 'anonymous',
  },
  plugins: [
    new HtmlRspackPlugin({
      chunks: ['app'],
      scriptLoading: 'module',
    }),
    new HtmlRspackPlugin({
      chunks: ['sync'],
      filename: 'sync.html',
      scriptLoading: 'module',
    }),
    new SubresourceIntegrityPlugin(),
  ],
};
