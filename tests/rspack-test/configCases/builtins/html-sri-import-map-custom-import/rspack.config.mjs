import { HtmlRspackPlugin, SubresourceIntegrityPlugin } from '@rspack/core';

export default {
  target: 'web',
  entry: {
    vendor: { import: './vendor.js', runtime: 'runtime' },
    app: { import: './app.js', dependOn: 'vendor' },
  },
  output: {
    module: true,
    filename: '[name].mjs',
    chunkFilename: '[name].mjs',
    importFunctionName: '__import__',
    publicPath: '/',
    crossOriginLoading: 'anonymous',
  },
  plugins: [
    new HtmlRspackPlugin({ chunks: ['app'], scriptLoading: 'module' }),
    new SubresourceIntegrityPlugin(),
  ],
};
