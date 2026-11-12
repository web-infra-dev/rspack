import { defineConfig, definePlugin } from '@rspack/cli';
import { NormalModule } from '@rspack/core';

export default defineConfig({
  optimization: {
    splitChunks: {
      minSize: 1,
    },
  },
  plugins: [
    definePlugin((compiler) => {
      compiler.hooks.compilation.tap('CheckNestedBlocks', (compilation) => {
        compilation.hooks.finishModules.tap('CheckNestedBlocks', (modules) => {
          const entry = [...modules].find(
            (module) =>
              module instanceof NormalModule &&
              module.rawRequest === './index.js',
          );
          expect(entry?.blocks).toHaveLength(1);
          expect(entry?.blocks[0].blocks).toHaveLength(3);
        });
      });
    }),
  ],
});
