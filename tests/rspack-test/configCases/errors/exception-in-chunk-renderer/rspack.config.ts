import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';

class ThrowsExceptionInRender {
  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap('ThrowsException', (compilation) => {
      // @ts-expect-error Rspack does not expose the webpack mainTemplate hooks.
      compilation.mainTemplate.hooks.requireExtensions.tap(
        'ThrowsException',
        () => {
          throw new Error('Test exception');
        },
      );
    });
  }
}

export default defineConfig({
  plugins: [new ThrowsExceptionInRender()],
});
