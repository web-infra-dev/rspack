import { container } from '@rspack/core';

const { ModuleFederationPlugin } = container;

/** @type {import("@rspack/core").Configuration} */
export default {
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
