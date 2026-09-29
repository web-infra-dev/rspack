import type { Compiler } from '@rspack/core';
import { defineConfig } from '@rspack/cli';
import assert from 'node:assert';

class ErrorPlugin {
  apply(compiler: Compiler) {
    compiler.hooks.thisCompilation.tap('DummyPlugin', (compilation) => {
      let error = new Error('error test');
      compilation.errors.push(error);
      let tempError = [...compilation.errors];
      assert(tempError.length === 1);
    });
  }
}

export default defineConfig({
  stats: 'errors-warnings',
  plugins: [new ErrorPlugin()],
});
