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
  cache: { type: 'persistent' },
  experiments: {
    newCache: { module: true, loader: false, codeGeneration: false },
  },
  optimization: { concatenateModules: false },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.done.tap('BuildMetaRecoveryTest', (stats) => {
          expect(stats.toJson({ all: false, errors: true }).errors).toEqual([]);
          expectAsync(stats.compilation, buildIndex < 2);
          buildIndex++;
        });
      },
    }),
  ],
});
