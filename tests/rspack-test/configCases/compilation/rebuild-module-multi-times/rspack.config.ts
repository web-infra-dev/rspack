import { defineConfig } from '@rspack/cli';
import type { Compiler, Module } from '@rspack/core';

const pluginName = 'plugin';

class Plugin {
  apply(compiler: Compiler) {
    let initial = true;
    compiler.hooks.compilation.tap(pluginName, (compilation) => {
      compilation.hooks.finishModules.tapPromise(
        pluginName,
        async (modules) => {
          const modulesToRebuild = [...modules];
          if (initial) {
            initial = false;

            const results = await Promise.all(
              modulesToRebuild.map((m) => {
                return new Promise<Module | null>((resolve, reject) => {
                  compilation.rebuildModule(m, (err, module) => {
                    if (err) {
                      reject(err);
                    } else {
                      resolve(module);
                    }
                  });
                });
              }),
            );

            // should compile success
            expect(results.length).toBe(4);
          }
        },
      );
    });
  }
}

export default defineConfig({
  module: {
    rules: [
      {
        test: /a\.js$/,
        use: [
          {
            loader: './loader.mjs',
          },
        ],
      },
    ],
  },
  plugins: [new Plugin()],
});
