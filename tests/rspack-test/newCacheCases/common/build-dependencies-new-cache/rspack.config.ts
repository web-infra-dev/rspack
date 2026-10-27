import { defineConfig, definePlugin } from '@rspack/cli';
import { getCacheStats } from '@rspack/test-tools';
import path from 'node:path';

const buildDependency = path.join(import.meta.dirname, './configs/index.js');
let compilerIndex = 0;

export default defineConfig({
  context: import.meta.dirname,
  cache: {
    type: 'persistent',
    buildDependencies: [buildDependency],
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.done.tapPromise('Test Plugin', async (stats) => {
          const [cacheStats] = await getCacheStats(stats);
          const { hits, misses } = cacheStats['Compilation/codeGeneration'];
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
