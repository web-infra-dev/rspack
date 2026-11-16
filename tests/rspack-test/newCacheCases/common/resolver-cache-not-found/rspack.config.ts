import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';

let buildIndex = 0;

export default defineConfig({
  plugins: [
    definePlugin((compiler) => {
      compiler.hooks.done.tap(
        'ResolverCacheNotFoundTest',
        ({ compilation }) => {
          const candidate = path.join(compiler.context, 'candidate.js');
          const present = buildIndex === 2 || buildIndex === 3;
          expect(compilation.fileDependencies.has(candidate)).toBe(present);
          if (!present) {
            expect(compilation.missingDependencies.has(candidate)).toBe(true);
          }
          buildIndex++;
        },
      );
    }),
  ],
});
