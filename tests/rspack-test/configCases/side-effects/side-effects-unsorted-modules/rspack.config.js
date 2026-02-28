/** @type {import("@rspack/core").Configuration} */

class ReorderModulesPlugin {
	constructor() { }

	apply(compiler) {
		compiler.hooks.compilation.tap("ReorderModulesPlugin", compilation => {
			compilation.hooks.seal.tap("ReorderModulesPlugin", () => {
				const sortedModules = Array.from(compilation.modules).sort((a, _b) =>
					a.request.includes("b.js") ? -1 : 1
				);
				compilation.modules = new Set(sortedModules);
			});
		});
	}
}

module.exports = {
	// We don't support modify the order of compilation.modules, but it's always unsorted
	// plugins: [new ReorderModulesPlugin()],
	optimization: {
		sideEffects: true
	}
};
