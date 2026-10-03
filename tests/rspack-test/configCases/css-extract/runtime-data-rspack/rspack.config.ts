import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig(
  ['css-ownership', '', 'name"with\\quotes'].map((uniqueName) => ({
    target: 'web',
    optimization: { chunkIds: 'named' },
    output: { uniqueName },
    module: {
      rules: [
        {
          test: /\.css$/,
          type: 'javascript/auto',
          use: [rspack.CssExtractRspackPlugin.loader, 'css-loader'],
        },
      ],
    },
    plugins: [
      new rspack.CssExtractRspackPlugin(),
      new rspack.DefinePlugin({ UNIQUE_NAME: JSON.stringify(uniqueName) }),
    ],
  })),
);
