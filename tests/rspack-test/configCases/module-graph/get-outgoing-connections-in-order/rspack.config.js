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
				const connection =
					compilation.moduleGraph.getConnection(entryDependency);
				const outgoingConnections =
					compilation.moduleGraph.getOutgoingConnectionsInOrder(
						connection.module
					);
				expect(normalize(outgoingConnections[0].module.request)).toBe(
					normalize(join(__dirname, "a.js"))
				);
				expect(normalize(outgoingConnections[1].module.request)).toBe(
					normalize(join(__dirname, "b.js"))
				);
				expect(normalize(outgoingConnections[2].module.request)).toBe(
					normalize(join(__dirname, "c.js"))
				);
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
