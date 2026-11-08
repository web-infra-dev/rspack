import { defineConfig } from '@rspack/cli';
import { type Compiler } from '@rspack/core';

var testPlugin = function (this: Compiler) {
  this.hooks.compilation.tap('TestPlugin', (compilation) => {
    compilation.hooks.finishModules.tapAsync(
      'TestPlugin',
      function (_modules, callback) {
        callback();
      },
    );
  });
};

export default defineConfig({
  plugins: [testPlugin],
});
