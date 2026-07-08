const { ModuleFederationPlugin } = require('@rspack/core').container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  optimization: {
    chunkIds: 'named',
    moduleIds: 'named',
  },
  output: {
    chunkFilename: '[name].js',
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'container',
      filename: 'container.js',
      library: { type: 'commonjs-module' },
      manifest: {
        // Windows-style separator, as `path.join` produces on Windows.
        filePath: 'custom\\path',
      },
      exposes: {
        './expose-a': './expose-a.js',
      },
    }),
  ],
};
