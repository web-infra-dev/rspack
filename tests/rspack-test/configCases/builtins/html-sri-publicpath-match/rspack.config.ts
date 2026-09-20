import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  mode: 'production',
  target: 'web',
  output: {
    publicPath: 'https://cdn.example.com/assets/',
    crossOriginLoading: 'anonymous',
  },
  node: {
    __dirname: false,
  },
  plugins: [
    new rspack.HtmlRspackPlugin({
      filename: 'index.html',
    }),
    new rspack.SubresourceIntegrityPlugin({
      hashFuncNames: ['sha384'],
      enabled: true,
    }),
  ],
});
