const { normalize } = require("path");

class Plugin {
	apply(compiler) {
		compiler.hooks.finishMake.tap("PLUGIN", compilation => {
			const entry = compilation.entries.get("main");
			const entryDependency = entry.dependencies[0];
			const entryModule = compilation.moduleGraph.getModule(entryDependency);
			expect(
				normalize(
					entryModule.libIdent({
						context: __dirname
					})
				)
			).toEqual("index.js");
		});
	}
}

/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	context: __dirname,
	entry: {
		main: "./index.js"
	},
	plugins: [new Plugin()]
};
