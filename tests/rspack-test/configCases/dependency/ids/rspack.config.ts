import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';

class Plugin {
  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap('Test', (compilation) => {
      compilation.hooks.finishModules.tap('Test', () => {
        const entry = compilation.entries.get('main');
        const entryDependency = entry!.dependencies[0];
        const entryModule = compilation.moduleGraph.getModule(entryDependency);
        const esmImportSpecifierDependency = entryModule!.dependencies.find(
          (d) => d.type === 'esm import specifier',
        );
        expect(esmImportSpecifierDependency?.ids).toContain('foo');
      });
    });
  }
}

export default defineConfig({
  entry: './index.js',
  optimization: {
    sideEffects: false,
  },
  plugins: [new Plugin()],
});
