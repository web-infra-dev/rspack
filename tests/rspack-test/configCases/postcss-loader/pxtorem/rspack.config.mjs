import { fileURLToPath } from 'node:url';

/** @type {import("@rspack/core").Configuration} */
export default {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  target: 'web',
  node: false,
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [
          {
            loader: 'postcss-loader',
            options: {
              postcssOptions: {
                plugins: [
                  fileURLToPath(import.meta.resolve('postcss-pxtorem')),
                ],
              },
            },
          },
        ],
        type: 'css/auto',
        generator: {
          exportsOnly: false,
        },
      },
    ],
  },
};
