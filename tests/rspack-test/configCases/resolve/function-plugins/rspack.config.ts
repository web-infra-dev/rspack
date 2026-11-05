import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';

let pluginExecutionCounter = 0;

export default defineConfig({
  plugins: [
    {
      apply(compiler: Compiler) {
        compiler.hooks.done.tap('TestPlugin', () => {
          expect(pluginExecutionCounter).toBeGreaterThan(0);
        });
      },
    },
  ],
  resolve: {
    // @ts-expect-error Rspack does not support resolver plugins yet.
    plugins: [
      {
        apply() {
          pluginExecutionCounter += 1;
        },
      },
      () => {
        pluginExecutionCounter += 1;
      },
    ],
  },
});
