const { normalize } = require("path");

const PLUGIN_NAME = "Test";

class Plugin {
	/**
	 * @param {import("@rspack/core").Compiler} compiler
	 */
	apply(compiler) {
		compiler.hooks.compilation.tap(PLUGIN_NAME, compilation => {
			compilation.hooks.finishModules.tap(PLUGIN_NAME, () => {
				const moduleGraph = compilation.moduleGraph;
				const entry = Array.from(compilation.entries.values())[0];
				const entryDependency = entry.dependencies[0];
				const entryConnection = moduleGraph.getConnection(entryDependency);
				const entryModule = entryConnection.module;

				// Get outgoing connections from entry module
				const outgoingConnections =
					moduleGraph.getOutgoingConnections(entryModule);

				// Find the connection to "used.js"
				const usedConnection = outgoingConnections.find(
					c => c.module && normalize(c.module.request).includes("used.js")
				);
				expect(usedConnection).toBeTruthy();

				// Test getActiveState on outgoing connection returns true (boolean)
				const outgoingState = moduleGraph.getActiveState(
					usedConnection,
					undefined
				);
				expect(outgoingState).toBe(true);
				expect(typeof outgoingState).toBe("boolean");

				// Get incoming connections for "used.js" module
				const usedModule = usedConnection.module;
				const incomingConnections =
					moduleGraph.getIncomingConnections(usedModule);
				expect(incomingConnections.length).toBeGreaterThan(0);

				// Verify all incoming connections to "used.js" are active
				for (const connection of incomingConnections) {
					const state = moduleGraph.getActiveState(connection, undefined);
					expect(state).toBe(true);
					expect(typeof state).toBe("boolean");
					expect(connection.originModule).toBeTruthy();
				}
			});
		});
	}
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	node: {
		__dirname: false,
		__filename: false
	},
	plugins: [new Plugin()]
};
