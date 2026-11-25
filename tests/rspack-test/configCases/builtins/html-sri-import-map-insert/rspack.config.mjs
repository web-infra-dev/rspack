import { HtmlRspackPlugin, SubresourceIntegrityPlugin } from '@rspack/core';

export default {
  target: 'web',
  output: {
    module: true,
    filename: '[name].mjs',
    crossOriginLoading: 'anonymous',
  },
  plugins: [
    new HtmlRspackPlugin({
      filename: 'pages/index.html',
      scriptLoading: 'module',
      templateContent:
        '<!DOCTYPE html><html><head><link rel="modulepreload" href="/vendor/external.mjs" nonce="test-nonce"></head><body></body></html>',
    }),
    new SubresourceIntegrityPlugin(),
  ],
};
