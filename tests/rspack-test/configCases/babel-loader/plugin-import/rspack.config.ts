import { defineConfig } from '@rspack/cli';

export default defineConfig({
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
});
