export default {
  module: {
    rules: [
      {
        test: /value\.js$/,
        use: () => [
          { loader: './loader.js', parallel: { maxWorkers: 1 }, options: {} },
        ],
      },
    ],
  },
};
