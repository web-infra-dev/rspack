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
    ...[
      ['pages/nonce.html', '/vendor/external.mjs', ''],
      [
        'pages/nonce-unrelated-preload.html',
        '/vendor/external.mjs',
        ' nonce="preload-nonce"',
      ],
      [
        'pages/nonce-bundled-preload.html',
        '../main.mjs',
        ' nonce="preload-nonce"',
      ],
    ].map(
      ([filename, href, attributes]) =>
        new HtmlRspackPlugin({
          filename,
          inject: false,
          templateContent: `<!DOCTYPE html><html><head>
            <link rel="modulepreload" href="${href}"${attributes}>
          </head><body>
            <script type="module" src="../main.mjs" nonce="module-nonce"></script>
          </body></html>`,
        }),
    ),
    new SubresourceIntegrityPlugin(),
  ],
};
