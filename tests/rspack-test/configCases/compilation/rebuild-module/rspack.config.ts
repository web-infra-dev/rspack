import { defineConfig } from '@rspack/cli';
import { type Compiler, type Module, NormalModule } from '@rspack/core';

const pluginName = 'plugin';

class Plugin {
  apply(compiler: Compiler) {
    let initial = true;
    compiler.hooks.compilation.tap(pluginName, (compilation) => {
      compilation.hooks.finishModules.tapPromise(
        pluginName,
        async (modules) => {
          const oldModule = [...modules].find(
            (item) =>
              item instanceof NormalModule && item.resource.endsWith('a.js'),
          );
          if (!oldModule) {
            throw new Error('module not found');
          }
          if (initial) {
            initial = false;

            expect(oldModule.originalSource()?.source().includes('a = 1')).toBe(
              true,
            );

            const newModule = await new Promise<Module>((res, rej) => {
              compilation.rebuildModule(oldModule, function (err, m) {
                if (err) {
                  rej(err);
                } else {
                  res(m!);
                }
              });
            });

            expect(newModule.originalSource()?.source().includes('a = 2')).toBe(
              true,
            );
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
