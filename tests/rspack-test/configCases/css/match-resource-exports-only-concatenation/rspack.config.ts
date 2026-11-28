import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  mode: 'production',
  devtool: false,
  optimization: {
    concatenateModules: true,
    minimize: false,
    usedExports: true,
  },
  module: {
    rules: [
      {
        oneOf: [
          {
            test: /\.no-id\.module\.css$/,
            type: 'css/auto',
            generator: {
              exportsOnly: true,
              esModule: true,
              localIdentName: '[local]-[hash:6]',
            },
          },
          {
            test: /\.global\.css$/,
            type: 'css/global',
            generator: {
              exportsOnly: true,
              esModule: true,
              localIdentName: '[id]-[local]',
            },
          },
          {
            test: /\.css$/,
            type: 'css/auto',
            generator: {
              exportsOnly: true,
              esModule: true,
              localIdentName: '[id]-[local]',
            },
          },
        ],
      },
    ],
  },
});
