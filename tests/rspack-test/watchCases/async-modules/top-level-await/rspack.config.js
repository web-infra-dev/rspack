/** @type {import('@rspack/core').Configuration} */
module.exports = [
  {
    externals: {
      fs: 'node-commonjs fs',
    },
    entry: {
      case1: './case1/index.js',
    },
    output: {
      filename: 'case1.js',
    },
  },
  {
    externals: {
      fs: 'node-commonjs fs',
    },
    entry: {
      case2: './case2/index.js',
    },
    output: {
      filename: 'case2.js',
    },
  },
  {
    externals: {
      fs: 'node-commonjs fs',
    },
    entry: {
      case3: './case3/index.js',
    },
    output: {
      filename: 'case3.js',
    },
  },
];
