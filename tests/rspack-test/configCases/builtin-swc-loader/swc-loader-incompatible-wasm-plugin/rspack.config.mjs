import { rspack } from '@rspack/core';
import path from 'node:path';
import sources from 'webpack-sources';
import { fileURLToPath } from 'node:url';

const { RawSource } = sources;

/** @type {import('@rspack/cli').Configuration} */
const config = {
  context: import.meta.dirname,
  entry: {
    main: './src/index.jsx',
  },
  resolve: {
    extensions: ['...', '.jsx'],
    alias: {
      '@swc/helpers': path.dirname(
        fileURLToPath(import.meta.resolve('@swc/helpers/package.json')),
      ),
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
            sourceMaps: true,
            detectSyntax: 'auto',
            jsc: {
              target: 'es5',
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
                cacheRoot: import.meta.dirname + '/.swc',
                plugins: [
                  [
                    import.meta.dirname + '/node_modules/swc-wasm-plugin',
                    {
                      exclude: ['error'],
                    },
                  ],
                ],
              },
            },
          },
        },
        type: 'javascript/auto',
      },
      {
        test: /\.(png|svg|jpg)$/,
        type: 'asset/resource',
      },
      {
        test: /\.css$/,
        type: 'css/auto',
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
    {
      // Replace all assets with empty content to avoid evaluation that causes errors
      apply(compiler) {
        compiler.hooks.compilation.tap('_', (compilation) => {
          compilation.hooks.processAssets.tap('_', (assets) => {
            let names = Object.keys(assets);
            names.forEach((name) => {
              assets[name] = new RawSource('');
            });
          });
        });
      },
    },
  ],
};
export default config;
