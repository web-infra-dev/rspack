import type { Compiler } from '@rspack/core';
import { defineConfig } from '@rspack/cli';
import assert from 'node:assert';

class Plugin {
  apply(compiler: Compiler) {
    let count = 0;
    compiler.hooks.compilation.tap('test', (compilation) => {
      assert(typeof compilation.hooks.afterProcessAssets !== 'undefined');
      compilation.hooks.afterProcessAssets.tap(
        'should-emit-should-works',
        (assets) => {
          assert(typeof assets !== 'undefined');
          assert(typeof assets['bundle0.js'] !== 'undefined');
          count += 1;
        },
      );
    });

    compiler.hooks.done.tap('check', () => {
      assert(count === 1);
    });
  }
}

export default defineConfig({
  context: import.meta.dirname,
  plugins: [new Plugin()],
});
