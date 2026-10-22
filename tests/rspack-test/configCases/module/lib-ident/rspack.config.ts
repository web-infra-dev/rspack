import { normalize } from 'node:path';
import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';

class Plugin {
  apply(compiler: Compiler) {
    compiler.hooks.finishMake.tap('PLUGIN', (compilation) => {
      const entry = compilation.entries.get('main');
      const entryDependency = entry!.dependencies[0];
      const entryModule = compilation.moduleGraph.getModule(entryDependency);
      expect(
        normalize(
          entryModule!.libIdent({
            context: import.meta.dirname,
          })!,
        ),
      ).toEqual('index.js');
    });
  }
}

export default defineConfig({
  context: import.meta.dirname,
  entry: {
    main: './index.js',
  },
  plugins: [new Plugin()],
});
