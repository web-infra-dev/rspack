import { defineConfig } from '@rspack/cli';
import type { Compilation, CompilationParams, Compiler } from '@rspack/core';
import { DefinePlugin } from '@rspack/core';

declare const currentWatchStep: { step?: string };

export default defineConfig({
  cache: true,
  plugins: [
    (compiler: Compiler) => {
      const base = {
        DEFINE: '{}',
        // @ts-expect-error Rspack does not support DefinePlugin.runtimeValue yet.
        RUN: DefinePlugin.runtimeValue(() => +(currentWatchStep.step || 0), []),
      };
      const defines = [
        {
          ...base,
          'DEFINE.A': 0,
          'DEFINE.B': 2,
        },
        {
          // change
          ...base,
          'DEFINE.A': 1,
          'DEFINE.B': 2,
        },
        {
          // add
          ...base,
          'DEFINE.A': 1,
          'DEFINE.B': 2,
          'DEFINE.C': 3,
        },
        {
          // remove
          ...base,
          'DEFINE.A': 1,
          'DEFINE.C': 3,
        },
      ];
      compiler.hooks.compilation.tap('rspack.config', (...args) => {
        const plugin = new DefinePlugin(defines[+(currentWatchStep.step || 0)]);
        plugin.apply({
          hooks: {
            compilation: {
              tap: (
                _name: string,
                fn: (...args: [Compilation, CompilationParams]) => void,
              ) => {
                fn(...args);
              },
            },
          },
        } as unknown as Compiler);
      });
    },
  ],
});
