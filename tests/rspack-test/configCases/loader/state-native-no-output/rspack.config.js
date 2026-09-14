module.exports = {
  module: {
    rules: [
      {
        test: /input\.js$/,
        use: [
          './verify.js',
          'builtin:test-no-finish-loader',
          './passthrough.js',
        ],
      },
    ],
  },
};
