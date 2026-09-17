import { strict as assert } from 'node:assert';

const pluginName = 'plugin';

class Plugin {
  apply(compiler) {
    let called = 0;
    compiler.hooks.compilation.tap(pluginName, (compilation) => {
      compilation.hooks.chunkAsset.tap(pluginName, (chunk, name) => {
        let files = [...chunk.files];
        assert(files.includes('bundle0.js'));
        called++;
      });
    });
    compiler.hooks.done.tap(pluginName, (stats) => {
      let json = stats.toJson();
      assert(json.errors.length === 0, `${json.errors}`);
      assert(called === 1);
    });
  }
}

/**@type {import("@rspack/core").Configuration}*/
export default {
  context: import.meta.dirname,
  module: {
    rules: [],
  },
  plugins: [new Plugin()],
};
