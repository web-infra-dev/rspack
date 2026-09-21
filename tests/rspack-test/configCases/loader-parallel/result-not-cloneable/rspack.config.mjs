export default {
  module: {
    rules: [
      {
        test: /lib\.js$/,
        use: [
          { loader: './loader.mjs', parallel: true, cache: false, options: {} },
        ],
      },
    ],
  },
};
