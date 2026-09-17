/** @type {import("@rspack/core").Configuration} */
export default [
  {
    target: 'web',
    mode: 'development',
    module: {
      generator: {
        'css/auto': {
          esModule: false,
        },
      },
      rules: [
        {
          test: /\.css$/,
          type: 'css/auto',
        },
      ],
    },
  },
  {
    target: 'node',
    mode: 'development',
    module: {
      generator: {
        'css/auto': {
          esModule: false,
        },
      },
      rules: [
        {
          test: /\.css$/,
          type: 'css/auto',
        },
      ],
    },
  },
];
