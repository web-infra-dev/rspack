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
    new HtmlRspackPlugin({
      filename: 'pages/nonce.html',
      inject: false,
      templateContent: `<!DOCTYPE html><html><head>
        <link rel="modulepreload" href="/vendor/external.mjs">
      </head><body>
        <script type="module" src="../main.mjs" nonce="module-nonce"></script>
      </body></html>`,
    }),
    new SubresourceIntegrityPlugin(),
  ],
};
