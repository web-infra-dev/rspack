import { HtmlRspackPlugin, SubresourceIntegrityPlugin } from '@rspack/core';

const baseURLs = {
  'absolute.html': 'https://cdn.example.test/assets/',
  'relative.html': '../assets/nested/..',
  'root-relative.html': '/assets/',
  'scheme-relative.html': '//cdn.example.test:443/assets/',
  'collision.html': './assets/',
  'normalized-collision.html': './assets/a/..',
};

export default {
  target: 'web',
  output: {
    module: true,
    filename: 'assets/bundle.mjs',
    chunkFilename: 'bundle.mjs',
    publicPath: './',
    crossOriginLoading: 'anonymous',
  },
  plugins: [
    ...Object.entries(baseURLs).map(
      ([filename, baseURL]) =>
        new HtmlRspackPlugin({
          filename,
          scriptLoading: 'module',
          templateContent: `<!DOCTYPE html><html><head>
            <script type="importmap">{
              "imports": {"external": "./external.mjs"},
              "scopes": {"./scope/": {"external": "./scoped.mjs"}},
              "integrity": {"./external.mjs": "sha384-external"}
            }</script>
            <base href="${baseURL}">
          </head><body></body></html>`,
        }),
    ),
    new HtmlRspackPlugin({
      filename: 'authored.html',
      publicPath: '/',
      scriptLoading: 'module',
      templateContent: `<!DOCTYPE html><html><head>
        <script type="importmap">{
          "integrity": {
            "/assets/../bundle.mjs": "sha384-first",
            "./bundle.mjs": "sha384-authored"
          }
        }</script>
      </head><body></body></html>`,
    }),
    new SubresourceIntegrityPlugin(),
  ],
};
