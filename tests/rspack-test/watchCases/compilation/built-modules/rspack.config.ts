import { defineConfig, definePlugin } from '@rspack/cli';
import { type NormalModule } from '@rspack/core';

let firstRun = true;

export default defineConfig({
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.compilation.tap('test', (compilation) => {
          compilation.hooks.seal.tap('test', () => {
            const builtModules = Array.from(compilation.builtModules).map(
              (m) => (m as NormalModule).rawRequest,
            );
            builtModules.sort();
            if (firstRun) {
              expect(builtModules).toEqual(['./foo', './index.js']);
              firstRun = false;
            } else {
              expect(builtModules).toEqual(['./foo']);
            }
          });
        });
      },
    }),
  ],
});
