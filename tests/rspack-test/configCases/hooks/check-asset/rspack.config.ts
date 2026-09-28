import { strict as assert } from 'node:assert';
import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';

const pluginName = 'plugin';

class Plugin {
  apply(compiler: Compiler) {
    let called = 0;
    compiler.hooks.compilation.tap(pluginName, (compilation) => {
      compilation.hooks.chunkAsset.tap(pluginName, (chunk) => {
        let files = [...chunk.files];
        assert(files.includes('bundle0.js'));
        called++;
      });
    });
    compiler.hooks.done.tap(pluginName, (stats) => {
      let json = stats.toJson();
      assert(json.errors?.length === 0, `${json.errors}`);
      assert(called === 1);
    });
  }
}

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [],
  },
  plugins: [new Plugin()],
});
