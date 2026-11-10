import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';

let buildIndex = 0;

export default defineConfig({
  mode: 'development',
  experiments: {
    newCache: {
      resolver: true,
      module: false,
      codeGeneration: false,
      devtool: false,
      loader: false,
      minimize: false,
    },
  },
  plugins: [
    definePlugin((compiler) => {
      compiler.hooks.done.tap('ResolverCacheNotFoundTest', (stats) => {
        const candidate = path.join(compiler.context, 'candidate.js');
        const present = buildIndex === 2 || buildIndex === 3;
        const { compilation } = stats;
        expect(compilation.fileDependencies.has(candidate)).toBe(present);
        if (!present) {
          expect(compilation.missingDependencies.has(candidate)).toBe(true);
        }
        if (buildIndex % 2 === 1) {
          const counters = stats.toJson({ all: false, cacheInfo: true })
            .cacheInfo!['|ResolverCache|normal'];
          expect(counters.hits).toBeGreaterThan(0);
          expect(counters.misses).toBe(0);
          expect(counters.reused).toBe(counters.hits);
          expect(counters.stores).toBe(0);
        }
        buildIndex++;
      });
    }),
  ],
});
