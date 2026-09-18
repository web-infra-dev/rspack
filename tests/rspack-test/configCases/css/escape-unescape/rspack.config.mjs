/** @type {import("@rspack/core").Configuration[]} */
export default [
  {
    externals: {
      path: 'node-commonjs path',
    },
    target: 'web',
    mode: 'development',

    module: {
      rules: [
        {
          test: /\.css$/,
          type: 'css/auto',
        },
      ],
    },
  },
  {
    externals: {
      path: 'node-commonjs path',
    },
    target: 'web',
    mode: 'production',

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
