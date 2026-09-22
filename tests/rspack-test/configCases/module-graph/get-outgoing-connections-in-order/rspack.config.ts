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
        const outgoingConnections =
          compilation.moduleGraph.getOutgoingConnectionsInOrder(
            connection.module!,
          );
        expect(
          normalize((outgoingConnections[0].module as NormalModule).request),
        ).toBe(normalize(join(import.meta.dirname, 'a.js')));
        expect(
          normalize((outgoingConnections[1].module as NormalModule).request),
        ).toBe(normalize(join(import.meta.dirname, 'b.js')));
        expect(
          normalize((outgoingConnections[2].module as NormalModule).request),
        ).toBe(normalize(join(import.meta.dirname, 'c.js')));
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
