import { normalize, join } from 'node:path';

const PLUGIN_NAME = 'Test';

class Plugin {
  /**
   * @param {import("@rspack/core").Compiler} compiler
   */
  apply(compiler) {
    compiler.hooks.compilation.tap(PLUGIN_NAME, (compilation) => {
      compilation.hooks.finishModules.tap(PLUGIN_NAME, () => {
        const fooModule = Array.from(compilation.modules.values()).find(
          (module) =>
            normalize(module.request) ===
            normalize(join(import.meta.dirname, 'foo.js')),
        );
        const issuer = compilation.moduleGraph.getIssuer(fooModule);
        expect(normalize(issuer.request)).toBe(
          normalize(join(import.meta.dirname, 'index.js')),
        );
      });
    });
  }
}

/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'web',
  node: {
    __dirname: false,
    __filename: false,
  },
  plugins: [new Plugin()],
};
