import assert from 'node:assert';
import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  module: { rules: [{ test: /\.js$/, sideEffects: false }] },
  optimization: { providedExports: true },
  plugins: [
    definePlugin((compiler) => {
      compiler.hooks.finishMake.tapAsync(
        'testcase',
        (compilation, callback) => {
          const resource = path.join(compiler.context, 'barrel.js');
          const module = [...compilation.modules].find(
            (module) => module.nameForCondition() === resource,
          );
          assert(module, 'Barrel module was not built');
          compilation.rebuildModule(module, (error) => callback(error));
        },
      );
    }),
  ],
});
