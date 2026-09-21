export default {
  module: {
    rules: [
      {
        test: /lib\.js$/,
        use: [
          {
            loader: './worker-loader.mjs',
            parallel: true,
            cache: false,
            options: {},
          },
          { loader: './seed-loader.mjs', cache: false },
        ],
      },
    ],
  },
};
