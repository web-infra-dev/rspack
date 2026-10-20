import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig(
  [false, true].map((lazyCompilation) => ({
    mode: 'development' as const,
    lazyCompilation: lazyCompilation
      ? { entries: false, imports: true }
      : false,
    module: {
      rules: [{ test: /(?:named|default|forwarded)\.js$/, sideEffects: false }],
    },
    optimization: { usedExports: false as const, sideEffects: 'flag' as const },
    plugins: [
      definePlugin((compiler) => {
        if (lazyCompilation) {
          rspack.lazyCompilationMiddleware(compiler);
        }
        compiler.hooks.done.tap('CheckLazyBarrelScope', (stats) => {
          const modules = [...stats.compilation.modules].map((m) =>
            path.basename(m.identifier()),
          );
          // A local default stops star lookup unless lazy compilation expands
          // the barrel. An explicitly reexported default still bypasses its barrel.
          expect(
            modules.filter((name) => name.endsWith('-deferred.js')).sort(),
          ).toEqual(
            lazyCompilation ? ['default-deferred.js', 'named-deferred.js'] : [],
          );
        });
      }),
    ],
  })),
);
