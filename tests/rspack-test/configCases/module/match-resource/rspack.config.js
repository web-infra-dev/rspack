const { resolve, normalize } = require("path");

class Plugin {
	apply(compiler) {
		compiler.hooks.finishMake.tap("PLUGIN", compilation => {
			const entry = compilation.entries.get("main");
			const entryDependency = entry.dependencies[0];
			const entryModule = compilation.moduleGraph.getModule(entryDependency);
			expect(normalize(entryModule.matchResource)).toEqual(
				resolve(__dirname, "index.js")
			);
		});
	}
}

/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	context: __dirname,
	entry: {
		main: "./index.js!=!./loader"
	},
	plugins: [new Plugin()]
};
