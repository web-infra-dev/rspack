const path = require('path');
const { ModuleFederationPlugin } = require('@rspack/core').container;

const implementation = require.resolve('@module-federation/runtime-tools', {
  paths: [path.dirname(require.resolve('@rspack/core/package.json'))],
});

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  optimization: {
    chunkIds: 'named',
    moduleIds: 'named',
    splitChunks: {
      cacheGroups: {
        shared: {
          test: /shared/,
          name: 'shared',
          chunks: 'all',
          enforce: true,
        },
      },
    },
  },
  output: {
    filename: '[name].js',
    chunkFilename: '[name].js',
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'container',
      filename: 'container.js',
      library: { type: 'commonjs-module' },
      implementation,
      manifest: true,
      exposes: {
        './expose-a': {
          import: './expose-a.js',
          name: '__federation_expose_expose-a',
        },
        './expose-b': {
          import: './expose-b.js',
          name: '__federation_expose_expose-b',
        },
      },
    }),
  ],
};
