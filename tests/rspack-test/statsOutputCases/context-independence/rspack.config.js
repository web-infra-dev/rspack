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
  // Load-bearing for cross-OS snapshot stability, do not remove: pnpm links
  // packages with symlinks on POSIX but with junctions on Windows, and
  // junctions are not reported as symlinks to the resolver, so the resolved
  // babel-loader path keeps the `node_modules/...` spelling on Windows while
  // POSIX canonicalizes it into the `.pnpm` store. The two spellings yield
  // different deterministic module/chunk ids (e.g. async chunk `270` vs
  // `191`), which end up in emitted file names where hash normalization
  // cannot paper over them. Skipping symlink resolution keeps the loader
  // path spelling identical on every platform.
  resolveLoader: {
    symlinks: false,
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
