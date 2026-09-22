import { defineConfig } from '@rspack/cli';
import type { Compiler, NormalModule } from '@rspack/core';
import { resolve, normalize } from 'node:path';

class Plugin {
  apply(compiler: Compiler) {
    compiler.hooks.finishMake.tap('PLUGIN', (compilation) => {
      const entry = compilation.entries.get('main')!;
      const entryDependency = entry.dependencies[0];
      const entryModule = compilation.moduleGraph.getModule(
        entryDependency,
      ) as NormalModule;
      expect(normalize(entryModule.matchResource!)).toEqual(
        resolve(import.meta.dirname, 'index.js'),
      );
    });
  }
}

export default defineConfig({
  context: import.meta.dirname,
  entry: {
    main: './index.js!=!./loader.mjs',
  },
  plugins: [new Plugin()],
});
