import assert from 'node:assert/strict';
import { defineConfig, definePlugin } from '@rspack/cli';
import { NormalModule } from '@rspack/core';

export default defineConfig({
  mode: 'development',
  plugins: [
    definePlugin((compiler) => {
      let step = 0;
      compiler.hooks.compilation.tap('CheckNestedBlocks', (compilation) => {
        const nested = step++ !== 1;
        compilation.hooks.finishModules.tap('CheckNestedBlocks', (modules) => {
          const entry = [...modules].find(
            (module) =>
              module instanceof NormalModule &&
              module.rawRequest === './index.js',
          );
          assert(entry, 'entry module not found');
          expect(entry.blocks).toHaveLength(1);
          expect(entry.blocks[0].blocks).toHaveLength(nested ? 1 : 0);
        });
      });
    }),
  ],
});
