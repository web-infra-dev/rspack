const { normalize } = require('path');
const {
  CssExtractRspackPlugin,
  TRANSITIVE_ONLY,
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
        const entry = Array.from(compilation.entries.values())[0];
        const entryDependency = entry.dependencies[0];
        const entryConnection = moduleGraph.getConnection(entryDependency);
        const entryModule = entryConnection.module;

        const outgoingConnections =
          moduleGraph.getOutgoingConnections(entryModule);

        // Find CSS module (processed by CssExtractRspackPlugin)
        const cssConnection = outgoingConnections.find(
          (c) =>
            c.module && normalize(c.module.request).includes('style.css'),
        );
        expect(cssConnection).toBeTruthy();

        // CSS module's outgoing connections (CssDependency) should have
        // TransitiveOnly state
        const cssModule = cssConnection.module;
        const cssOutgoing = moduleGraph.getOutgoingConnections(cssModule);
        const transitiveConnections = cssOutgoing.filter(
          (conn) => conn.getActiveState(undefined) === TRANSITIVE_ONLY,
        );
        expect(transitiveConnections.length).toBeGreaterThan(0);
        for (const conn of transitiveConnections) {
          expect(typeof conn.getActiveState(undefined)).toBe('symbol');
        }
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
