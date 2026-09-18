export default {
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /lib\.js/,
        use: [
          {
            loader: './unclonable.mjs',
            options: {
              notclonable() {},
            },
          },
          {
            loader: './loader-in-worker.mjs',
            parallel: { maxWorkers: 2 },
            options: {},
          },
        ],
      },
    ],
  },
};
