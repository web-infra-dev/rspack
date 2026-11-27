import { HtmlRspackPlugin, SubresourceIntegrityPlugin } from '@rspack/core';

export default {
  target: 'web',
  entry: { main: './index.js', other: './other.js' },
  output: {
    module: true,
    filename: ({ chunk }) =>
      chunk.name === 'other'
        ? 'entries/other.mjs'
        : 'entries/[name].[contenthash:8].mjs',
    chunkFilename: 'chunks/[name].[contenthash:8].mjs',
    publicPath: 'https://cdn.example.test/assets/',
    crossOriginLoading: 'anonymous',
  },
  plugins: [
    new HtmlRspackPlugin({
      chunks: ['main'],
      scriptLoading: 'module',
      templateContent: `<!DOCTYPE html><html><head>
				<script type="importmap" nonce="test-nonce">{
					"imports": {"external": "/vendor/external.mjs"},
					"scopes": {"/vendor/": {"shared": "/vendor/shared.mjs"}},
					"integrity": {"/vendor/external.mjs": "sha384-external"}
				}</script>
				<link rel="modulepreload" href="/vendor/external.mjs">
			</head><body></body></html>`,
    }),
    new HtmlRspackPlugin({
      chunks: ['other'],
      filename: 'other.html',
      scriptLoading: 'module',
    }),
    new HtmlRspackPlugin({
      chunks: ['other'],
      filename: 'manual-other.html',
      inject: false,
      scriptLoading: 'module',
      templateContent:
        '<!DOCTYPE html><html><head>' +
        '<script type="importmap">{}</script>' +
        '<script type="module">import("https://cdn.example.test/assets/entries/other.mjs");</script>' +
        '</head><body></body></html>',
    }),
    new HtmlRspackPlugin({
      chunks: ['other'],
      filename: 'versioned-other.html',
      hash: true,
      scriptLoading: 'module',
      templateContent: `<!DOCTYPE html><html><head>
        <script type="module">import("https://cdn.example.test/assets/entries/other.mjs");</script>
      </head><body></body></html>`,
    }),
    new SubresourceIntegrityPlugin(),
  ],
  optimization: {
    concatenateModules: false,
    splitChunks: {
      cacheGroups: {
        shared: {
          test: /shared\.js$/,
          name: 'shared',
          chunks: 'async',
          enforce: true,
        },
      },
    },
  },
};
