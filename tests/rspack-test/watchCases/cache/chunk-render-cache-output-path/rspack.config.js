let compilationCount = 0;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  target: 'web',
  entry: {
    first: './first.js',
    trigger: './trigger.js',
  },
  output: {
    publicPath: '',
    filename: (pathData) => {
      if (pathData.chunk?.name !== 'first') {
        return '[name].js';
      }
      return compilationCount > 1 ? 'deep/nested/first.js' : 'first.js';
    },
  },
  cache: {
    type: 'memory',
  },
  module: {
    parser: {
      javascript: {
        url: 'new-url-relative',
        importMeta: false,
      },
    },
  },
  optimization: {
    runtimeChunk: 'single',
    concatenateModules: false,
    inlineExports: false,
    splitChunks: false,
  },
  incremental: true,
  plugins: [
    (compiler) => {
      compiler.hooks.thisCompilation.tap('testcase', () => {
        compilationCount += 1;
      });
    },
  ],
};
