import { defineConfig } from '@rspack/cli';
import {
  type Compiler,
  type ConcatenatedModule,
  type NormalModule,
} from '@rspack/core';
import { normalize, join } from 'node:path';

const PLUGIN_NAME = 'Test';

class Plugin {
  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap(PLUGIN_NAME, (compilation) => {
      compilation.hooks.afterProcessAssets.tap(PLUGIN_NAME, () => {
        const entry = Array.from(compilation.entries.values())[0];
        const entryDependency = entry.dependencies[0];

        const module = compilation.moduleGraph.getModule(
          entryDependency,
        ) as ConcatenatedModule;
        expect(module.modules.length).toBe(2);

        const resolvedModule = compilation.moduleGraph.getResolvedModule(
          entryDependency,
        ) as NormalModule;
        expect(normalize(resolvedModule.request)).toBe(
          normalize(join(import.meta.dirname, 'index.js')),
        );
      });
    });
  }
}

export default defineConfig({
  target: 'web',
  node: {
    __dirname: false,
    __filename: false,
  },
  plugins: [new Plugin()],
  optimization: {
    concatenateModules: true,
    // inlineExports will inline foo.js into index.js, so there is no module.modules
    inlineExports: false,
  },
});
