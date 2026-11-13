import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';
import { NormalModule } from '@rspack/core';

export default defineConfig({
  mode: 'development',
  devtool: 'source-map',
  plugins: [
    definePlugin((compiler) => {
      compiler.hooks.finishMake.tapPromise(
        'RebuildEntry',
        async (compilation) => {
          const entry = [...compilation.modules].find(
            (module) =>
              module instanceof NormalModule &&
              path.basename(module.resource) === 'index.js',
          );
          await new Promise<void>((resolve, reject) => {
            compilation.rebuildModule(entry!, (error) => {
              if (error) reject(error);
              else resolve();
            });
          });
        },
      );
    }),
  ],
});
