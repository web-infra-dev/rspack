import { defineConfig } from '@rspack/cli';
import { type Compiler } from '@rspack/core';
import path from 'node:path';
import fs from 'node:fs';

class ShouldRebuildPlugin {
  compileCount = 0;
  apply(compiler: Compiler) {
    const targetFile = path.resolve(compiler.context, './index.js');

    compiler.hooks.done.tap(ShouldRebuildPlugin.name, () => {
      // After first compilation, touch the file to trigger a rebuild
      if (this.compileCount === 0) {
        const now = new Date();
        fs.utimes(targetFile, now, now, (err) => {
          if (err) {
            console.error('Error updating file timestamps:', err);
            return;
          }
          // Touch file to trigger rebuild
        });
      }
      this.compileCount++;
    });
  }
}

const config = {
  plugins: [new ShouldRebuildPlugin()],
  watchOptions: {
    aggregateTimeout: 1000,
  },
};

export default defineConfig(config);
