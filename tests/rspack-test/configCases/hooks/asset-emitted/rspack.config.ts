import type { Compiler } from '@rspack/core';
import { defineConfig } from '@rspack/cli';
import { strict as assert } from 'node:assert';

const pluginName = 'plugin';

class Plugin {
  apply(compiler: Compiler) {
    let hasMainJs = false;
    compiler.hooks.assetEmitted.tap(pluginName, (filename, info) => {
      if (filename === 'bundle0.js') {
        assert(info.targetPath.includes('bundle0.js'));
        assert(info.content.toString().includes('expect(3).toBe(3)'));
        hasMainJs = true;
      }
    });
    compiler.hooks.done.tap(pluginName, () => {
      assert(hasMainJs);
    });
  }
}

export default defineConfig({
  context: import.meta.dirname,
  plugins: [new Plugin()],
});
