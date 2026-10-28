import { defineConfig, definePlugin } from '@rspack/cli';
import path from 'node:path';

const buildDependency = path.join(import.meta.dirname, './configs/index.js');
let compilerIndex = 0;

export default defineConfig({
  context: import.meta.dirname,
  experiments: {
    newCache: true,
  },
  cache: {
    type: 'persistent',
    buildDependencies: [buildDependency],
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.done.tap('Test Plugin', (stats) => {
          const cacheStats = stats.toJson({
            all: false,
            cacheInfo: true,
          }).cacheInfo!;
          const { hits, misses } =
            cacheStats[`${compiler.compilerPath}|Compilation/codeGeneration`];
          const total = hits + misses;
          expect(total).toBeGreaterThan(0);

          if (compilerIndex === 1 || compilerIndex === 4) {
            expect(hits).toBe(total);
          } else {
            expect(hits).toBe(0);
          }
          compilerIndex++;
        });
      },
    }),
  ],
});
