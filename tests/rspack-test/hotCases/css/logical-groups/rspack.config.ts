import path from 'node:path';
import { defineConfig } from '@rspack/cli';

let generation = -1;
export default defineConfig({
  output: { cssChunkFilename: '[name].css' },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.thisCompilation.tap('MoveChunks', () => {
          generation++;
        });
      },
    },
  ],
  module: { rules: [{ test: /\.css$/, type: 'css/auto' }] },
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
