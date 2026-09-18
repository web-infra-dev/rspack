import path from 'node:path';
import rspack from '@rspack/core';

/** @type {rspack.Configuration} */
export default {
  context: import.meta.dirname,
  entry: {
    main: './src/index.js',
  },
  devtool: false,
  mode: 'development',
  resolve: {
    alias: {
      'shared-lib': path.resolve(import.meta.dirname, 'src/shared-lib'),
    },
  },
  plugins: [
    new rspack.HtmlRspackPlugin({ template: './src/index.html' }),
    new rspack.container.ModuleFederationPlugin({
      name: 'host',
      shared: {
        'shared-lib': {
          singleton: true,
          eager: false,
          requiredVersion: false,
        },
      },
    }),
  ],
  devServer: {
    hot: true,
  },
};
