const path = require("path");

class Plugin {
	apply(compiler) {
		compiler.hooks.finishMake.tap("PLUGIN", compilation => {
			const entry = compilation.entries.get("main");
			const entryDependency = entry.dependencies[0];
			const entryModule = compilation.moduleGraph.getModule(entryDependency);
			expect(entryModule.buildInfo.loaded).toBe(true);
		});
	}
}

/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	module: {
		rules: [
			{
				test: /\.js/,
				use: [path.join(__dirname, "loader.js")]
			}
		]
	},
	plugins: [new Plugin()]
};
