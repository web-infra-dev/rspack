import path from 'node:path';
import { strict as assert } from 'node:assert';

const pluginName = 'plugin';

class Plugin {
  apply(compiler) {
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

/**@type {import("@rspack/core").Configuration}*/
export default {
  context: import.meta.dirname,
  plugins: [new Plugin()],
};
