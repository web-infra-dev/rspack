const path = require("path");

class Plugin {
	apply(compiler) {
		compiler.hooks.compilation.tap("Test", compilation => {
			compilation.hooks.processAssets.tap("Test", () => {
				const chunk = Array.from(compilation.chunks)[0];
				const modules = compilation.chunkGraph.getChunkModules(chunk);
				expect(modules[0].resource).toBe(path.join(__dirname, "index.js"));
			});
		});
	}
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	node: false,
	entry: {
		main: "./index.js"
	},
	output: {
		filename: "[name].js"
	},
	optimization: {
		sideEffects: false
	},
	plugins: [new Plugin()]
};
