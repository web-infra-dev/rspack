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
          expect(entry.blocks).toHaveLength(1);
          expect(entry.blocks[0].blocks).toHaveLength(1);
          expect(entry.blocks[0].blocks[0].blocks).toHaveLength(0);
        });
      });
    }),
  ],
});
