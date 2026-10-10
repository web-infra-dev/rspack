import path from 'node:path';
import { rspack } from '@rspack/core';
import { defineConfig } from '@rspack/cli';

let generation = -1;
export default defineConfig({
  plugins: [
    new rspack.CssExtractRspackPlugin({ filename: '[name].css' }),
    {
      apply(compiler) {
        compiler.hooks.thisCompilation.tap('MoveChunks', () => {
          generation++;
        });
      },
    },
  ],
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'javascript/auto',
        use: [rspack.CssExtractRspackPlugin.loader, 'css-loader'],
      },
    ],
  },
  optimization: {
    moduleIds: 'named',
    chunkIds: 'named',
    concatenateModules: false,
    splitChunks: {
      chunks: 'all',
      minSize: 0,
      cacheGroups: {
        default: false,
        defaultVendors: false,
        moving: {
          test: /shared-(loaded|unloaded)\.css$/,
          name: (module) =>
            `${path.basename(module.nameForCondition()!, '.css')}-${generation}`,
          enforce: true,
        },
      },
    },
  },
});
