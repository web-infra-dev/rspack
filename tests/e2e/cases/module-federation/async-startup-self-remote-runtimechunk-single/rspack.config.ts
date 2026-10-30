import type { Configuration, Compiler } from '@rspack/core';
import { rspack } from '@rspack/core';

export default {
  context: import.meta.dirname,
  entry: './src/index.js',
  mode: 'development',
  devtool: false,
  resolve: {
    extensions: ['...'],
  },
  output: {
    filename: 'static/js/[name].js',
    chunkFilename: 'static/js/[name].js',
  },
  optimization: {
    runtimeChunk: 'single',
    chunkIds: 'named',
    moduleIds: 'named',
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
          './RemoteModule': './src/RemoteModule.js',
        },
        remotes: {
          remote: `remote@http://localhost:${port}/remoteEntry.js`,
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
} satisfies Configuration;
