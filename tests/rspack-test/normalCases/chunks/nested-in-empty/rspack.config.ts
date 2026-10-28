import assert from 'node:assert/strict';
import { defineConfig, definePlugin } from '@rspack/cli';
import { NormalModule } from '@rspack/core';

export default defineConfig({
  plugins: [
    definePlugin((compiler) => {
      compiler.hooks.compilation.tap('CheckNestedBlocks', (compilation) => {
        compilation.hooks.finishModules.tap('CheckNestedBlocks', (modules) => {
          const entry = [...modules].find(
            (module) =>
              module instanceof NormalModule &&
              module.resource.replace(/\\/g, '/').endsWith('/index.js'),
          );
          assert(entry, 'entry module not found');
          let blocks = entry.blocks;
          for (let depth = 0; depth < 4; depth++) {
            expect(blocks).toHaveLength(1);
            blocks = blocks[0].blocks;
          }
          expect(blocks).toHaveLength(0);
        });
      });
    }),
  ],
});
