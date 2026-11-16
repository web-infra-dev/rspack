import type { Compiler, Module } from '@rspack/core';
import { defineConfig } from '@rspack/cli';
import { strict } from 'node:assert';

const pluginName = 'plugin';

class Plugin {
  apply(compiler: Compiler) {
    let identifiers: string[] = [];
    compiler.hooks.compilation.tap(pluginName, (compilation) => {
      const collectModule = (m: Module) => {
        identifiers.push(m.identifier());
      };
      compilation.hooks.buildModule.tap(pluginName, collectModule);
      compilation.hooks.stillValidModule.tap(pluginName, collectModule);
    });
    compiler.hooks.done.tap(pluginName, () => {
      strict(identifiers.some((i) => i.endsWith('index.js')));
      strict(identifiers.some((i) => i.endsWith('a.js')));
    });
  }
}

export default defineConfig({
  context: import.meta.dirname,
  plugins: [new Plugin()],
});
