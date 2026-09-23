import { defineConfig, definePlugin } from '@rspack/cli';
import { Compiler } from '@rspack/core';

export default defineConfig({
  optimization: {
    minimize: true,
    minimizer: [
      definePlugin({
        apply(compiler) {
          expect(compiler).toBeInstanceOf(Compiler);
        },
      }),

      definePlugin(function (compiler) {
        expect(compiler).toBe(this);
        expect(compiler).toBeInstanceOf(Compiler);
      }),
    ],
  },
});
