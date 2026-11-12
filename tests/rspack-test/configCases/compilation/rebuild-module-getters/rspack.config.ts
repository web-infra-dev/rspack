import { defineConfig } from '@rspack/cli';
import { NormalModule, type Compiler } from '@rspack/core';

const pluginName = 'plugin';

class Plugin {
  apply(compiler: Compiler) {
    let initial = true;
    compiler.hooks.compilation.tap(pluginName, (compilation) => {
      compilation.hooks.finishModules.tapPromise(
        pluginName,
        async (modules) => {
          if (!initial) {
            return;
          }
          initial = false;
          const oldModule = [...modules].find(
            (item) =>
              item instanceof NormalModule && item.resource.endsWith('a.js'),
          );
          if (!oldModule) {
            throw new Error('module not found');
          }
          await new Promise((res, rej) => {
            compilation.rebuildModule(oldModule, function (err, m) {
              if (err) {
                rej(err);
              } else {
                res(m);
              }
            });
          });
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
