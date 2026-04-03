const { normalize } = require('path');
const { CssExtractRspackPlugin, TRANSITIVE_ONLY } = require('@rspack/core');

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

        // Find the connection to "used.js"
        const usedConnection = outgoingConnections.find(
          (c) => c.module && normalize(c.module.request).includes('used.js'),
        );
        expect(usedConnection).toBeTruthy();

        // Active connection should return true (boolean)
        const outgoingState = usedConnection.getActiveState(undefined);
        expect(outgoingState).toBe(true);
        expect(typeof outgoingState).toBe('boolean');

        // Incoming connections to "used.js" should all be active
        const usedModule = usedConnection.module;
        const incomingConnections =
          moduleGraph.getIncomingConnections(usedModule);
        expect(incomingConnections.length).toBeGreaterThan(0);
        for (const connection of incomingConnections) {
          const state = connection.getActiveState(undefined);
          expect(state).toBe(true);
          expect(connection.originModule).toBeTruthy();
        }
      });

      // Test TransitiveOnly in processAssets phase where exports info is available
      compilation.hooks.processAssets.tap(PLUGIN_NAME, () => {
        const moduleGraph = compilation.moduleGraph;

        // Walk all modules to find CssDependency connections with TransitiveOnly state
        let foundTransitiveOnly = false;
        for (const module of compilation.modules) {
          const outgoing = moduleGraph.getOutgoingConnections(module);
          for (const conn of outgoing) {
            const state = conn.getActiveState(undefined);
            if (state === TRANSITIVE_ONLY) {
              foundTransitiveOnly = true;
              expect(typeof state).toBe('symbol');
            }
          }
        }
        expect(foundTransitiveOnly).toBe(true);
      });
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
