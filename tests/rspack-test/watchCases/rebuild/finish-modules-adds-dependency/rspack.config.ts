import { resolve, join } from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';
import { NormalModule, type Compiler, type LoaderContext } from '@rspack/core';

// Rebuild the entry module from a finishModules tap and make the rebuild add
// a new dependency, mirroring content-rewriting plugins. Later incremental
// watch rebuilds must still see a consistent module graph and output.
class RebuildEntryPlugin {
  name = 'RebuildEntryPlugin';

  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap(this.name, (compilation) => {
      let shouldReplace = false;
      NormalModule.getCompilationHooks(compilation).loader.tap(
        this.name,
        (loaderContext: LoaderContext & { shouldReplace?: boolean }) => {
          loaderContext.shouldReplace = shouldReplace;
        },
      );
      compilation.hooks.finishModules.tapPromise(this.name, async (modules) => {
        const entryResource = resolve(
          join(compiler.options.context, 'index.js'),
        );
        const module = Array.from(modules).find(
          (m) => m instanceof NormalModule && m.resource === entryResource,
        );
        if (!module) {
          throw new Error('entry module not found');
        }
        // Skip when the cached module already carries the replaced build.
        if (module.buildInfo._isReplaced) return;
        shouldReplace = true;
        try {
          await new Promise<void>((resolvePromise) => {
            compilation.rebuildModule(module, () => resolvePromise());
          });
        } finally {
          shouldReplace = false;
        }
      });
    });
  }
}

export default defineConfig({
  module: {
    rules: [
      {
        test: /index\.js$/,
        use: [resolve(import.meta.dirname, 'loader.mjs')],
      },
    ],
  },
  optimization: {
    concatenateModules: false,
  },
  plugins: [new RebuildEntryPlugin()],
});
