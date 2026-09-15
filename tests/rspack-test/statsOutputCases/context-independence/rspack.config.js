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
    filename: '[name]-[chunkhash].js',
    // Keep the emitted/cached asset status deterministic: all six child
    // compilations share one dist directory and the a/b pairs produce
    // identical files, so content comparison would race between "[emitted]"
    // and "[cached]".
    compareBeforeEmit: false,
  },
  resolve: {
    alias: {
      c: [
        path.resolve(__dirname, name, 'c'),
        path.resolve(__dirname, name, 'cc'),
      ],
    },
  },
  // pnpm links packages with symlinks on POSIX but with junctions on
  // Windows, and junctions are not reported as symlinks to the resolver,
  // so the resolved loader path keeps the `node_modules/babel-loader`
  // spelling on Windows while POSIX canonicalizes it into the `.pnpm`
  // store. The two spellings relativize to different module names, which
  // changes deterministic module/chunk ids and content hashes across
  // operating systems. Skipping symlink resolution keeps the resolved
  // loader path spelling identical on every platform.
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
