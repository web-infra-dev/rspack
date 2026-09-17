import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    rules: [
      {
        test: /\.less$/,
        use: [
          {
            loader: 'less-loader',
            options: {
              lessOptions: {
                paths: [
                  'node_modules',
                  path.resolve(import.meta.dirname, 'node_modules'),
                ],
              },
            },
          },
        ],
        type: 'css',
      },
    ],
  },
};
