const path = require('path');
const rspack = require('@rspack/core');

function config(version) {
  return {
    mode: 'development',
    target: 'node',
    context: __dirname,
    entry: {
      runtime: {
        import: './runtime.js',
        filename: 'runtime.[chunkhash].js',
      },
      main: {
        import: `./${version}/index.js`,
        dependOn: 'runtime',
      },
    },
    output: {
      path: path.resolve(__dirname, `dist/${version}`),
      filename: '[name].[fullhash:base64:8].js',
      chunkFilename: '[id].js',
      chunkLoading: 'require',
      hotUpdateChunkFilename: '[id].hot-update.js',
      hotUpdateMainFilename: 'hot-update.json',
    },
    optimization: {
      chunkIds: 'named',
      moduleIds: 'named',
      minimize: false,
      realContentHash: false,
    },
    plugins: [new rspack.HotModuleReplacementPlugin()],
  };
}

/** @type {import('@rspack/core').Configuration[]} */
module.exports = [config('version0'), config('version1')];
