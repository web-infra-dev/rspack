import { resolve, join } from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';
import { NormalModule, type LoaderContext } from '@rspack/core';

const testPlugin = definePlugin((compiler) => {
  compiler.hooks.compilation.tap('TestPlugin', (compilation) => {
    let shouldReplace = false;
    NormalModule.getCompilationHooks(compilation).loader.tap(
      'TestPlugin',
      (loaderContext: LoaderContext & { shouldReplace?: boolean }) => {
        loaderContext.shouldReplace = shouldReplace;
      },
    );
    compilation.hooks.finishModules.tapAsync(
      'TestPlugin',
      function (modules, callback) {
        const src = resolve(join(import.meta.dirname, 'other-file.js'));

        const module = Array.from(modules).find(
          (m) => m instanceof NormalModule && m.resource === src,
        );

        if (!module) {
          throw new Error('something went wrong');
        }

        // Check if already build the updated version
        // this will happen when using caching
        if (module.buildInfo._isReplaced) return callback();

        shouldReplace = true;
        compilation.rebuildModule(module, (err) => {
          shouldReplace = false;
          callback(err);
        });
      },
    );
  });
});

export default defineConfig({
  module: {
    rules: [
      {
        test: /other-file/,
        use: './loader.mjs',
      },
    ],
  },
  optimization: {
    concatenateModules: false,
  },
  plugins: [testPlugin],
});
