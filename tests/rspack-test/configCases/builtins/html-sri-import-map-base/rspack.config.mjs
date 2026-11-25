import { HtmlRspackPlugin, SubresourceIntegrityPlugin } from '@rspack/core';

const preload =
  '<link rel="modulepreload" href="/unrelated.mjs" nonce="preload-nonce">';
const inlineModule =
  '<script type="module">export const ready = "import map";</script>';

const baseURLs = {
  'absolute.html': 'https://cdn.example.test/assets/',
  'relative.html': '../assets/nested/..',
  'root-relative.html': '/assets/',
  'scheme-relative.html': '//cdn.example.test:443/assets/',
  'collision.html': './assets/',
  'normalized-collision.html': './assets/a/..',
  'preload-before-base.html': '/assets/',
  'preload-before-colliding-base.html': './assets/',
  'inline-before-base.html': './assets/',
  'absolute-deferred-before-base.html': 'https://cdn.example.test/assets/',
};

export default {
  target: 'web',
  output: {
    module: true,
    filename: 'main+entry.mjs',
    chunkFilename: '[name].mjs',
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
            ${filename.startsWith('preload-') ? preload : ''}
            ${filename === 'inline-before-base.html' ? inlineModule : ''}
            ${filename === 'absolute-deferred-before-base.html' ? `<script type="module">import('${baseURL}bundle.mjs');</script>` : ''}
            <base href="${baseURL}">
          </head><body></body></html>`,
        }),
    ),
    new HtmlRspackPlugin({
      filename: 'same-basename-before-base.html',
      publicPath: '/app/',
      scriptLoading: 'module',
      templateContent: `<!DOCTYPE html><html><head>
        <script type="importmap">{}</script>
        <link rel="modulepreload" href="https://vendor.example.test/widgets/main+entry.mjs">
        <base href="https://cdn.example.test/assets/">
      </head><body></body></html>`,
    }),
    new HtmlRspackPlugin({
      filename: 'generated-before-base.html',
      scriptLoading: 'module',
      templateContent: `<!DOCTYPE html><html><head>
        ${preload}
        <base href="/assets/">
      </head><body></body></html>`,
    }),
    new HtmlRspackPlugin({
      filename: 'base-after-module.html',
      scriptLoading: 'module',
      inject: false,
      templateContent: `<!DOCTYPE html><html><head>
        <script type="importmap">{}</script>
        <script src="./main+entry.mjs" type="module"></script>
        <base href="/assets/">
      </head><body></body></html>`,
    }),
    new HtmlRspackPlugin({
      filename: 'deferred-import-before-map.html',
      scriptLoading: 'module',
      templateContent: `<!DOCTYPE html><html><head>
        <script type="module" id="deferred-module">import('./bundle.mjs');</script>
        <script type="importmap">{}</script>
      </head><body></body></html>`,
    }),
    new HtmlRspackPlugin({
      filename: 'deferred-import-before-two-maps.html',
      inject: false,
      scriptLoading: 'module',
      templateContent: `<!DOCTYPE html><html><head>
        <script type="module">import('./main+entry.mjs');</script>
        <script type="importmap">{"imports":{"one":"./one.mjs"}}</script>
        <script type="importmap" nonce="author-nonce">{"imports":{"two":"./two.mjs"}}</script>
      </head><body></body></html>`,
    }),
    new HtmlRspackPlugin({
      filename: 'absolute-deferred-without-injection.html',
      inject: false,
      scriptLoading: 'module',
      templateContent:
        '<!DOCTYPE html><html><head>' +
        '<script type="importmap">{"imports":{"external":"./external.mjs"}}</script>' +
        '<script type="module">import("https://cdn.example.test/assets/main+entry.mjs");</script>' +
        '<base href="https://cdn.example.test/assets/">' +
        '</head><body></body></html>',
    }),
    new HtmlRspackPlugin({
      filename: 'different-query-module.html',
      scriptLoading: 'module',
      hash: true,
      templateContent:
        '<!DOCTYPE html><html><head>' +
        '<script type="module" src="./main%2Bentry.mjs?variant=vendor"></script>' +
        '<script type="importmap">{}</script>' +
        '</head><body></body></html>',
    }),
    new HtmlRspackPlugin({
      filename: 'rewritten-module-src.html',
      publicPath: '/',
      scriptLoading: 'module',
    }),
    {
      apply(compiler) {
        compiler.hooks.compilation.tap('RewriteModuleSrc', (compilation) => {
          HtmlRspackPlugin.getCompilationHooks(
            compilation,
          ).afterTemplateExecution.tap('RewriteModuleSrc', (data) => {
            if (data.outputName === 'rewritten-module-src.html') {
              for (const tag of [...data.headTags, ...data.bodyTags]) {
                if (tag.attributes.type === 'module') {
                  tag.attributes.src = '/served' + tag.attributes.src;
                }
              }
            }
            return data;
          });
        });
      },
    },
    new HtmlRspackPlugin({
      filename: 'different-query-preload.html',
      scriptLoading: 'module',
      hash: true,
      templateContent: `<!DOCTYPE html><html><head>
        <link rel="modulepreload" href="./main%2Bentry.mjs?variant=vendor">
        <script type="importmap">{}</script>
      </head><body></body></html>`,
    }),
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
    ...[
      ['multiple-authored.html', undefined, preload],
      [
        'multiple-authored-other-port.html',
        '//cdn.example.test:8443/app/',
        '<link rel="modulepreload" href="https://cdn.example.test:9443/app/main%2Bentry.mjs">',
      ],
      [
        'multiple-authored-relative-ports.html',
        '//cdn.example.test:443/app/',
        '<link rel="modulepreload" href="//cdn.example.test:80/app/main%2Bentry.mjs">',
      ],
    ].map(
      ([filename, publicPath, preload]) =>
        new HtmlRspackPlugin({
          filename,
          publicPath,
          scriptLoading: 'module',
          templateContent: `<!DOCTYPE html><html><head>
            <script type="importmap">{"imports":{"external":"./external.mjs"}}</script>
            ${preload}
            <script type="importmap" nonce="authored-nonce">{
              "integrity": {"./bundle.mjs": "sha384-authored"}
            }</script>
          </head><body></body></html>`,
        }),
    ),
    new SubresourceIntegrityPlugin(),
  ],
};
