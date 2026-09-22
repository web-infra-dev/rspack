import { defineConfig } from '@rspack/cli';
import { type Compiler, NormalModule } from '@rspack/core';
import { normalize, join } from 'node:path';

const PLUGIN_NAME = 'Test';

class Plugin {
  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap(PLUGIN_NAME, (compilation) => {
      compilation.hooks.finishModules.tap(PLUGIN_NAME, () => {
        const fooModule = Array.from(compilation.modules.values()).find(
          (module) =>
            module instanceof NormalModule &&
            normalize(module.request) ===
              normalize(join(import.meta.dirname, 'foo.js')),
        )!;
        const issuer = compilation.moduleGraph.getIssuer(
          fooModule,
        ) as NormalModule;
        expect(normalize(issuer.request)).toBe(
          normalize(join(import.meta.dirname, 'index.js')),
        );
      });
    });
  }
}

export default defineConfig({
  target: 'web',
  node: {
    __dirname: false,
    __filename: false,
  },
  plugins: [new Plugin()],
});
