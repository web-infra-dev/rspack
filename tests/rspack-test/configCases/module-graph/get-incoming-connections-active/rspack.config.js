const { normalize } = require('path');
const {
  CssExtractRspackPlugin,
  ModuleGraphConnection,
} = require('@rspack/core');

const PLUGIN_NAME = 'Test';

class Plugin {
  /**
   * @param {import("@rspack/core").Compiler} compiler
   */
  apply(compiler) {
    compiler.hooks.compilation.tap(PLUGIN_NAME, (compilation) => {
      // Test active connections in finishModules phase
      compilation.hooks.finishModules.tap(PLUGIN_NAME, () => {
        const moduleGraph = compilation.moduleGraph;
        const entry = Array.from(compilation.entries.values())[0];
        const entryDependency = entry.dependencies[0];
        const entryConnection = moduleGraph.getConnection(entryDependency);
        const entryModule = entryConnection.module;

        const outgoingConnections =
          moduleGraph.getOutgoingConnections(entryModule);

        const usedConnections = outgoingConnections.filter(
          (c) => c.module && normalize(c.module.request).includes('used.js'),
        );
        // The value import is active; the pure module's side-effect import is not.
        const outgoingStates = usedConnections.map((connection) =>
          connection.getActiveState(undefined),
        );
        expect(new Set(outgoingStates)).toEqual(new Set([false, true]));

        const usedModule = usedConnections[0].module;
        const incomingConnections =
          moduleGraph.getIncomingConnections(usedModule);
        expect(incomingConnections.length).toBeGreaterThan(0);
        for (const connection of incomingConnections) {
          const state = connection.getActiveState(undefined);
          expect(typeof state).toBe('boolean');
          expect(connection.originModule).toBeTruthy();
        }
        expect(
          new Set(
            incomingConnections.map((connection) =>
              connection.getActiveState(undefined),
            ),
          ),
        ).toEqual(new Set(outgoingStates));
      });

      const checkTransitiveOnly = () => {
        const moduleGraph = compilation.moduleGraph;

        // Walk all modules to find CssDependency connections with TransitiveOnly state
        let foundTransitiveOnly = false;
        for (const module of compilation.modules) {
          const outgoing = moduleGraph.getOutgoingConnections(module);
          for (const conn of outgoing) {
            const state = conn.getActiveState(undefined);
            if (state === ModuleGraphConnection.TRANSITIVE_ONLY) {
              foundTransitiveOnly = true;
              expect(typeof state).toBe('symbol');
            }
          }
        }
        expect(foundTransitiveOnly).toBe(true);
      };
      compilation.hooks.finishModules.tap(
        { name: PLUGIN_NAME, stage: 20 },
        checkTransitiveOnly,
      );
      compilation.hooks.processAssets.tap(PLUGIN_NAME, checkTransitiveOnly);
    });
  }
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'web',
  node: {
    __dirname: false,
    __filename: false,
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [CssExtractRspackPlugin.loader, 'css-loader'],
        type: 'javascript/auto',
      },
    ],
  },
  plugins: [new CssExtractRspackPlugin(), new Plugin()],
};
