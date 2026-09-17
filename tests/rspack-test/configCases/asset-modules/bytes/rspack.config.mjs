/** @type {import("@rspack/core").Configuration[]} */
export default [
  {
    name: 'web',
    mode: 'development',
    target: 'web',
    module: {
      rules: [
        {
          test: /\.(png|svg)$/,
          type: 'asset/bytes',
        },
        {
          test: /\.css$/,
          type: 'css/auto',
        },
      ],
    },
  },
  {
    name: 'node',
    mode: 'development',
    target: 'node',
    module: {
      rules: [
        {
          test: /\.(png|svg)$/,
          type: 'asset/bytes',
        },
        {
          test: /\.css$/,
          type: 'css/auto',
        },
      ],
    },
  },
  {
    name: 'universal',
    mode: 'development',
    target: ['web', 'node'],
    output: {
      module: true,
    },
    module: {
      rules: [
        {
          test: /\.(png|svg)$/,
          type: 'asset/bytes',
        },
        {
          test: /\.css$/,
          type: 'css/auto',
        },
      ],
    },
  },
];
