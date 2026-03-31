const { normalize, join } = require("path");

const PLUGIN_NAME = "Test";

class Plugin {
	/**
	 * @param {import("@rspack/core").Compiler} compiler
	 */
	apply(compiler) {
		compiler.hooks.compilation.tap(PLUGIN_NAME, compilation => {
			compilation.hooks.finishModules.tap(PLUGIN_NAME, () => {
				const entry = Array.from(compilation.entries.values())[0];
				const entryDependency = entry.dependencies[0];
				const entryConnection =
					compilation.moduleGraph.getConnection(entryDependency);
				const entryModule = entryConnection.module;

				// Get outgoing connections from entry module
				const outgoingConnections =
					compilation.moduleGraph.getOutgoingConnections(entryModule);

				// Find the connection to "used.js"
				const usedConnection = outgoingConnections.find(
					c => c.module && normalize(c.module.request).includes("used.js")
				);
				expect(usedConnection).toBeTruthy();

				// Test getActiveState on outgoing connection
				expect(usedConnection.getActiveState(undefined)).toBe(true);

				// Get incoming connections for "used.js" module
				const usedModule = usedConnection.module;
				const incomingConnections =
					compilation.moduleGraph.getIncomingConnections(usedModule);
				expect(incomingConnections.length).toBeGreaterThan(0);

				// Verify incoming connections are active via getActiveState
				for (const connection of incomingConnections) {
					const state = connection.getActiveState(undefined);
					expect(state).toBe(true);
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
