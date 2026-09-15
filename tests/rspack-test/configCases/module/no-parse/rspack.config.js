module.exports = [
  {
    entry: './index.js',
    externals: {
      'node:fs': 'node-commonjs node:fs',
    },
    module: {
      noParse: require.resolve('./not-parsed-a'),
    },
  },
  {
    entry: './index.js',
    externals: {
      'node:fs': 'node-commonjs node:fs',
    },
    module: {
      noParse: /not-parsed/,
    },
  },
  {
    entry: './index.js',
    externals: {
      'node:fs': 'node-commonjs node:fs',
    },
    module: {
      noParse(content) {
        return /not-parsed/.test(content);
      },
    },
  },
  {
    entry: './index.js',
    externals: {
      'node:fs': 'node-commonjs node:fs',
    },
    module: {
      noParse: /not-parsed/,
      rules: [{ test: /\.js$/, loader: 'builtin:swc-loader' }],
    },
  },
];
