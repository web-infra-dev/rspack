import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';

const pluginName = 'plugin';

class Plugin {
  apply(compiler: Compiler) {
    let called = false;
    compiler.hooks.compilation.tap(pluginName, (compilation) => {
      compilation.hooks.processAssets.tap(pluginName, () => {
        const mainChunk = Array.from(compilation.chunks).find(
          (chunk) => chunk.name === 'main',
        );
        const p = compilation.getPath('[contenthash]', {
          contentHash: 'xxx1',
          chunk: mainChunk,
        });
        called = true;
        expect(p).toBe('xxx1');
      });
    });
    compiler.hooks.done.tap(pluginName, (stats) => {
      let json = stats.toJson();
      expect(json.errors?.length === 0);
      expect(called).toBe(true);
    });
  }
}

export default defineConfig({
  context: import.meta.dirname,
  plugins: [new Plugin()],
});
