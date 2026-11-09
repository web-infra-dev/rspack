export default {
  module: {
    rules: [
      {
        test: /resource\.js$/,
        use: [
          './check-loader.js',
          {
            loader: './worker-loader.js',
            parallel: { maxWorkers: 2 },
            options: {},
          },
          'builtin:test-passthrough-loader',
          { loader: './worker-loader.js', parallel: true, options: {} },
          './seed-loader.js',
        ],
      },
    ],
  },
};
