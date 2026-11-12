import { defineConfig } from '@rspack/cli';

const common = defineConfig({
  externals: {
    path: 'node-commonjs path',
  },
  target: 'web',
  mode: 'development',
  devtool: false,
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
});

export default defineConfig([
  {
    ...common,
    output: {
      publicPath: 'auto',
      cssChunkFilename: 'bundle0/css/[name].css',
      assetModuleFilename: 'bundle0/assets/[name][ext]',
    },
  },
  {
    ...common,
    output: {
      publicPath: 'https://test.cases/path/',
      cssChunkFilename: 'bundle1/css/[name].css',
      assetModuleFilename: 'bundle1/assets/[name][ext]',
    },
  },
  {
    ...common,
    output: {
      cssChunkFilename: 'bundle2/css/[name].css',
    },
    module: {
      rules: [
        {
          test: /\.css$/,
          type: 'css/auto',
        },
        {
          test: /\.png$/i,
          type: 'asset/resource',
          generator: {
            filename: '[name][ext]',
            outputPath: 'bundle2/assets/',
            publicPath: 'https://test.cases/path/bundle2/assets/',
          },
        },
      ],
    },
  },
  {
    ...common,
    output: {
      cssChunkFilename: 'apps/desk/[name].css',
    },
    module: {
      rules: [
        {
          test: /\.css$/,
          type: 'css/auto',
        },
        {
          test: /\.png$/i,
          type: 'asset/resource',
          generator: {
            filename: 'img/[name][ext]',
            outputPath: 'apps/desk/',
            publicPath: './',
          },
        },
      ],
    },
  },
]);
