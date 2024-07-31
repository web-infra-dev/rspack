

const rspack = require('@rspack/core');
const path = require('path');
/** @type {import('@rspack/cli').Configuration} */
const config = {
  entry: {
    main: './src/index.jsx',
  },
  resolve: {
    extensions: ['...', '.jsx'],
    alias: {
      '@swc/helpers': path.dirname(require.resolve('@swc/helpers/package.json')),
    },
  },
  module: {
    rules: [
      {
        test: /\.(jsx|js)$/,
        use: {
          loader: 'builtin:swc-loader',
          options: {
            // Enable source map
            sourceMap: true,
            target: 'es5',
            jsc: {
              parser: {
                syntax: 'ecmascript',
                jsx: true,
              },
              externalHelpers: true,
              preserveAllComments: false,
              transform: {
                react: {
                  runtime: 'automatic',
                  pragma: 'React.createElement',
                  pragmaFrag: 'React.Fragment',
                  throwIfNamespace: true,
                  useBuiltins: false,
                },
              },
                experimental: {
									cacheRoot: __dirname + "/.swc",
									plugins: [
										[
											__dirname + "/node_modules/swc-wasm-plugin",
											{
												exclude: ["error"]
											}
										]
									]
								}
            },
          },
        },
        type: 'javascript/auto',
      },
      {
        test: /\.(png|svg|jpg)$/,
        type: 'asset/resource',
      },
    ],
  },
  optimization: {
    minimize: false, // Disabling minification because it takes too long on CI
  },
  plugins: [
    new rspack.HtmlRspackPlugin({
      template: './index.html',
    }),
  ],
  experiments: { css: true }
};
module.exports = config;

