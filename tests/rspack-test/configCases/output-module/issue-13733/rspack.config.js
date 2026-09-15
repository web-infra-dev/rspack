const rspack = require('@rspack/core');

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  mode: 'production',
  target: 'node14',
  entry: './index.js',
  output: {
    filename: '[name].mjs',
    module: true,
    chunkFormat: 'module',
    chunkLoading: 'import',
  },
  optimization: {
    mangleExports: 'size',
    minimize: false,
  },
  plugins: [
    new rspack.optimize.LimitChunkCountPlugin({
      maxChunks: 1,
    }),
  ],
  externals(ctx) {
    if (ctx.request?.startsWith('node:')) {
      return `module ${ctx.request}`;
    }
  },
};
