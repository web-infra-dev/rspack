import { defineConfig } from '@rspack/cli';
import { type Compiler, type NormalModule } from '@rspack/core';
import { normalize, join } from 'node:path';

const PLUGIN_NAME = 'Test';

class Plugin {
  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap(PLUGIN_NAME, (compilation) => {
      compilation.hooks.finishModules.tap(PLUGIN_NAME, () => {
        const entry = Array.from(compilation.entries.values())[0];
        const entryDependency = entry.dependencies[0];
        const connection =
          compilation.moduleGraph.getConnection(entryDependency)!;
        const outgoingConnection =
          compilation.moduleGraph.getOutgoingConnections(connection.module!)[0];
        expect(
          normalize((outgoingConnection.module as NormalModule).request),
        ).toBe(normalize(join(import.meta.dirname, 'foo.js')));
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
