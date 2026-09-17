/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    rules: [
      {
        test: /\.js$/,
        use: [
          {
            loader: 'babel-loader',
            options: {
              plugins: [
                [
                  'babel-plugin-import',
                  {
                    libraryName: 'antd',
                  },
                ],
              ],
            },
          },
        ],
        type: 'javascript/auto',
      },
    ],
  },
};
