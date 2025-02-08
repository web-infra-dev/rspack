const { normalize, join } = require("path");

const PLUGIN_NAME = "Test";

class Plugin {
	/**
	 * @param {import("@rspack/core").Compiler} compiler
	 */
	apply(compiler) {
		compiler.hooks.compilation.tap(PLUGIN_NAME, compilation => {
			compilation.hooks.optimizeModules.tap(PLUGIN_NAME, () => {
				const moduleA = Array.from(compilation.modules).find(
					module =>
						normalize(module.resource) === normalize(join(__dirname, "a.js"))
				);
				const exportsInfoA = compilation.moduleGraph.getExportsInfo(moduleA);
				expect(exportsInfoA.isModuleUsed("main")).toBe(true);
				expect(exportsInfoA.isUsed("main")).toBe(true);
				expect(exportsInfoA.getUsed("good", "main")).toBe(4);
				expect(exportsInfoA.getUsed("cool", "main")).toBe(0);

				const moduleB = Array.from(compilation.modules).find(
					module =>
						normalize(module.resource) === normalize(join(__dirname, "b.js"))
				);
				const exportsInfoB = compilation.moduleGraph.getExportsInfo(moduleB);
				expect(exportsInfoB.isModuleUsed("main")).toBe(false);
				expect(exportsInfoB.isUsed("main")).toBe(false);

				const moduleC = Array.from(compilation.modules).find(
					module =>
						normalize(module.resource) === normalize(join(__dirname, "c.js"))
				);
				const exportsInfoC = compilation.moduleGraph.getExportsInfo(moduleC);
				expect(exportsInfoC.isModuleUsed("main")).toBe(true);
				expect(exportsInfoC.isUsed("main")).toBe(false);
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
