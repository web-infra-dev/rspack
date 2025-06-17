const path = require("path");

class Plugin {
	apply(compiler) {
		compiler.hooks.finishMake.tap("PLUGIN", compilation => {
			const entry = compilation.entries.get("main");
			const entryDependency = entry.dependencies[0];
			const entryModule = compilation.moduleGraph.getModule(entryDependency);
			expect(entryModule.buildInfo.loaded).toBe(true);

			// known build info
			expect(Object.keys(entryModule.buildInfo.assets)).toContain("foo.txt");

			expect(entryModule.buildInfo.fileDependencies.size).toBe(1);
			expect(
				entryModule.buildInfo.fileDependencies.has(
					path.join(__dirname, "index.js")
				)
			).toBe(true);

			expect(entryModule.buildInfo.buildDependencies.size).toBe(1);
			expect(entryModule.buildInfo.buildDependencies.has("./build.txt")).toBe(
				true
			);

			expect(entryModule.buildInfo.contextDependencies.size).toBe(0);

			expect(entryModule.buildInfo.missingDependencies.size).toBe(0);
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
