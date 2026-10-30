import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';
import { rspack } from '@rspack/core';

export default defineConfig({
  context: import.meta.dirname,
  entry: './src/index.jsx',
  mode: 'development',
  devtool: false,
  resolve: {
    extensions: ['...', '.jsx'],
  },
  output: {
    filename: 'static/js/[name].js',
    chunkFilename: 'static/js/[name].js',
  },
  optimization: {
    chunkIds: 'named',
    moduleIds: 'named',
  },
  module: {
    rules: [
      {
        test: /\.jsx?$/,
        use: [
          {
            loader: 'builtin:swc-loader',
            options: {
              detectSyntax: 'auto',
              jsc: {
                transform: {
                  react: {
                    runtime: 'automatic',
                    development: true,
                  },
                },
              },
            },
          },
        ],
      },
    ],
  },
  plugins: [
    new rspack.HtmlRspackPlugin({ template: './src/index.html' }),
    function (compiler: Compiler) {
      // The E2E fixture assigns a numeric port before creating the compiler.
      const { port } = compiler.options.devServer as { port: number };
      new rspack.container.ModuleFederationPlugin({
        name: 'remote',
        filename: 'remoteEntry.js',
        exposes: {
          './RemoteComponent': './src/RemoteComponent.jsx',
        },
        remotes: {
          remote: `remote@http://localhost:${port}/remoteEntry.js`,
        },
        shared: {
          react: {},
          'react-dom': {},
        },
        experiments: {
          asyncStartup: true,
        },
      }).apply(compiler);
    },
  ],
  devServer: {
    hot: true,
    devMiddleware: {
      writeToDisk: true,
    },
  },
});
