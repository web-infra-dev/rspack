module.exports = {
  module: {
    rules: [
      {
        test: /m\d\.js$/,
        use: [
          { loader: './loader.js', parallel: { maxWorkers: 2 }, options: {} },
        ],
      },
    ],
  },
};
