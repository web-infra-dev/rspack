import path from 'node:path';
import { defineConfig } from '@rspack/cli';

let generation = -1;
export default defineConfig({
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.thisCompilation.tap('MoveChunks', () => {
          generation++;
        });
      },
    },
  ],
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
          test: /shared-(loaded|unloaded)\.js$/,
          name: (module) =>
            `${path.basename(module.nameForCondition()!, '.js')}-${generation}`,
          enforce: true,
        },
      },
    },
  },
});
