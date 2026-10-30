import { resolve, join } from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';
import { NormalModule, type Compiler, type LoaderContext } from '@rspack/core';

// The entry keeps its dependency during the cold build; on the next
// incremental build the changed dependency is rebuilt by the make phase
// (recording a module mutation for it) and only then does the finishModules
// rebuild drop the import, orphaning and removing the module behind it. The
// mutation log must not keep pointing at the removed module.
class RebuildEntryPlugin {
  name = 'RebuildEntryPlugin';

  apply(compiler: Compiler) {
    // Counts compilations across the compiler lifetime so the rewrite
    // first happens in an incremental (hot) build.
    let buildCount = 0;
    compiler.hooks.compilation.tap(this.name, (compilation) => {
      let shouldReplace = false;
      NormalModule.getCompilationHooks(compilation).loader.tap(
        this.name,
        (loaderContext: LoaderContext & { shouldReplace?: boolean }) => {
          loaderContext.shouldReplace = shouldReplace;
        },
      );
      compilation.hooks.finishModules.tapPromise(this.name, async (modules) => {
        buildCount++;
        // Only rewrite the entry from the second (incremental) build on,
        // so the make phase of that build has already recorded a module
        // mutation for the dependency the rebuild is about to remove.
        if (buildCount < 2) return;
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
