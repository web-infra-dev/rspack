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
          const target = [...modules].find(
            (item) =>
              item instanceof NormalModule && item.resource.endsWith('a.js'),
          );
          if (!target) {
            throw new Error('module not found');
          }
          if (initial) {
            initial = false;
            await new Promise<Module>((resolve, reject) => {
              compilation.rebuildModule(target, (err, module) => {
                if (err) {
                  reject(err);
                } else {
                  resolve(module!);
                }
              });
            });
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
