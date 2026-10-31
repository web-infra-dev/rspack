import { defineConfig } from '@rspack/cli';

export default defineConfig([
  {
    output: {
      filename: 'bundle0.[contenthash].a.js',
      assetModuleFilename: 'img/[name].a.[contenthash][ext]',
    },
    optimization: {
      // @ts-expect-error Rspack does not support size module IDs yet.
      moduleIds: 'size',
    },
    module: {
      rules: [
        {
          test: /\.jpg$/,
          type: 'asset/resource',
        },
      ],
    },
  },
  {
    output: {
      filename: 'bundle1.[contenthash].b.js',
      assetModuleFilename: 'img/[name].a.[contenthash][ext]',
    },
    optimization: {
      // @ts-expect-error Rspack does not support size module IDs yet.
      moduleIds: 'size',
    },
    module: {
      rules: [
        {
          test: /\.jpg$/,
          type: 'asset/resource',
        },
      ],
    },
  },
]);
