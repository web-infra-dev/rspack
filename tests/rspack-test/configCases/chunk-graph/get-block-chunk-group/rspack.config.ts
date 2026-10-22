import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';

class Plugin {
  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap('Test', (compilation) => {
      compilation.hooks.processAssets.tap('Test', () => {
        const entry = compilation.entries.get('main');
        const entryDependency = entry!.dependencies[0];
        const entryModule = compilation.moduleGraph.getModule(entryDependency);
        const block = entryModule!.blocks[0];
        const chunkGroup = compilation.chunkGraph.getBlockChunkGroup(block);
        expect(chunkGroup?.name).toBe('foo');
      });
    });
  }
}

export default defineConfig({
  target: 'web',
  node: false,
  entry: {
    main: './index.js',
  },
  output: {
    filename: '[name].js',
  },
  optimization: {
    sideEffects: false,
  },
  plugins: [new Plugin()],
});
