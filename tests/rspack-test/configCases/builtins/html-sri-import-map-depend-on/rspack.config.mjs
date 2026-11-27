import { HtmlRspackPlugin, SubresourceIntegrityPlugin } from '@rspack/core';

export default {
  target: 'web',
  entry: {
    shared: { import: './shared.js', runtime: 'runtime' },
    vendor: { import: './vendor.js', dependOn: 'shared' },
    app: { import: './app.js', dependOn: 'vendor' },
    other: { import: './other.js', dependOn: 'vendor' },
    sync: { import: './sync.js', dependOn: 'shared' },
    percentShared: {
      import: './percent-shared.js',
      filename: 'chunks%2Bv1/shared.mjs',
    },
    percentSync: {
      import: './percent-sync.js',
      dependOn: 'percentShared',
      filename: 'chunks%2Bv1/sync.mjs',
    },
  },
  output: {
    module: true,
    filename: 'chunks+v1/[name].mjs',
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
    new HtmlRspackPlugin({
      chunks: ['sync', 'percentSync'],
      filename: 'encoded-collision.html',
      scriptLoading: 'module',
    }),
    new SubresourceIntegrityPlugin(),
  ],
};
