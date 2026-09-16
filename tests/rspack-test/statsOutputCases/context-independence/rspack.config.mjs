import path from 'node:path';

/**
 * @param {string} name name
 * @param {string} devtool devtool
 * @returns {import("@rspack/core").Configuration} configuration
 */
const base = (name, devtool) => ({
  mode: 'production',
  devtool,
  module: {
    rules: [
      {
        test: /chunk/,
        loader: 'babel-loader',
        options: {},
      },
    ],
  },
  stats: {
    assets: true,
    modules: true,
    relatedAssets: true,
  },
  entry: {
    main: {
      import: './index',
      layer: 'my-layer',
    },
  },
  context: path.resolve(import.meta.dirname, name),
  output: {
    path: path.resolve(
      import.meta.dirname,
      `../../js/stats/context-independence/${devtool}-${name}`,
    ),
    filename: '[name]-[chunkhash].js',
  },
  resolve: {
    alias: {
      c: [
        path.resolve(import.meta.dirname, name, 'c'),
        path.resolve(import.meta.dirname, name, 'cc'),
      ],
    },
  },
  // Keep loader paths and deterministic ids consistent across POSIX symlinks and Windows junctions.
  resolveLoader: {
    symlinks: false,
  },
});

/** @type {import("@rspack/core").Configuration[]} */
export default [
  base('a', 'source-map'),
  base('b', 'source-map'),
  base('a', 'eval-source-map'),
  base('b', 'eval-source-map'),
  base('a', 'eval'),
  base('b', 'eval'),
];
