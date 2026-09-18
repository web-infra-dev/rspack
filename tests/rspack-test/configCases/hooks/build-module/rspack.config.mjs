import { strict } from 'node:assert';

const pluginName = 'plugin';

class Plugin {
  apply(compiler) {
    let identifiers = [];
    compiler.hooks.compilation.tap(pluginName, (compilation) => {
      compilation.hooks.buildModule.tap(pluginName, (m) => {
        identifiers.push(m.identifier());
      });
    });
    compiler.hooks.done.tap(pluginName, () => {
      strict(identifiers.some((i) => i.endsWith('index.js')));
      strict(identifiers.some((i) => i.endsWith('a.js')));
    });
  }
}

/**@type {import("@rspack/core").Configuration}*/
export default {
  context: import.meta.dirname,
  plugins: [new Plugin()],
};
