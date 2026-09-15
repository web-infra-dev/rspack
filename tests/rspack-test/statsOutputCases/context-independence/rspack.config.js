const path = require('path');

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
        // require.resolve yields the loader's realpath inside the pnpm
        // store on every OS, so deterministic module/chunk ids don't depend
        // on pnpm's symlink (POSIX) vs junction (Windows) layout.
        loader: require.resolve('babel-loader'),
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
  context: path.resolve(__dirname, name),
  output: {
    path: path.resolve(
      __dirname,
      `../../js/stats/context-independence/${devtool}-${name}`,
    ),
    filename: '[name]-[chunkhash].js',
  },
  resolve: {
    alias: {
      c: [
        path.resolve(__dirname, name, 'c'),
        path.resolve(__dirname, name, 'cc'),
      ],
    },
  },
});

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
  base('a', 'source-map'),
  base('b', 'source-map'),
  base('a', 'eval-source-map'),
  base('b', 'eval-source-map'),
  base('a', 'eval'),
  base('b', 'eval'),
];
