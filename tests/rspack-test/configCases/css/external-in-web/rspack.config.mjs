/** @type {import("@rspack/core").Configuration} */
export default [
  {
    target: 'web',
    optimization: {
      chunkIds: 'named',
    },

    module: {
      rules: [
        {
          test: /\.css$/,
          type: 'css/auto',
        },
      ],
    },
  },
];
