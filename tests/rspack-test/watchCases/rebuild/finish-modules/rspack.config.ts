import { resolve, join } from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';
import { NormalModule } from '@rspack/core';

// Rebuild the entry module from a finishModules tap, mirroring plugins that
// rewrite module content (e.g. virtual bootstrap modules) before the modules
// are finalized. The next incremental watch rebuild must still see a
// consistent module graph instead of panicking on stale dependency ids.
const testPlugin = definePlugin((compiler) => {
  compiler.hooks.compilation.tap('TestPlugin', (compilation) => {
    compilation.hooks.finishModules.tapPromise(
      'TestPlugin',
      async (modules) => {
        const entryResource = resolve(
          join(compiler.options.context, 'index.js'),
        );
        const module = Array.from(modules).find(
          (m) => m instanceof NormalModule && m.resource === entryResource,
        );
        if (!module) {
          throw new Error('entry module not found');
        }
        await new Promise<void>((resolvePromise) => {
          compilation.rebuildModule(module, () => resolvePromise());
        });
      },
    );
  });
});

export default defineConfig({
  optimization: {
    concatenateModules: false,
  },
  plugins: [testPlugin],
});
