import type { Compilation, Module } from '@rspack/core';
import { defineConfig, definePlugin } from '@rspack/cli';

let buildIndex = 0;

function expectAsync(compilation: Compilation, expected: boolean) {
  for (const filename of ['index.js', 'dep.js']) {
    const module = [...compilation.modules].find(
      (module: Module) =>
        (module as { resource?: string }).resource?.replace(/^.*[\\/]/, '') ===
        filename,
    );
    expect(module).toBeDefined();
    expect(compilation.moduleGraph.isAsync(module!)).toBe(expected);
  }
}

export default defineConfig({
  mode: 'development',
  incremental: true,
  cache: { type: 'persistent' },
  experiments: {
    newCache: { module: true, loader: false, codeGeneration: false },
  },
  optimization: { concatenateModules: false },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.done.tap('BuildMetaRecoveryTest', (stats) => {
          const failed = buildIndex === 2 || buildIndex === 3;
          expect(
            stats.toJson({ all: false, errors: true }).errors,
          ).toHaveLength(failed ? 1 : 0);
          expectAsync(stats.compilation, buildIndex < 4);
          buildIndex++;
        });
      },
    }),
  ],
});
