import { HtmlRspackPlugin, SubresourceIntegrityPlugin } from '@rspack/core';

export default {
  target: 'web',
  entry: { main: './index.js', other: './other.js' },
  output: {
    module: true,
    filename: 'entries/[name].[contenthash:8].mjs',
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
