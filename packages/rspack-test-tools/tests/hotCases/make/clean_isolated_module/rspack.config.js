const plugin = {
	name: "PLUGIN",
	apply(compiler) {
		let isFirst = true;
		compiler.hooks.compilation.tap("PLUGIN", compilation => {
			compilation.hooks.finishModules.tap("PLUGIN", modules => {
				let hasModuleB = false;
				let hasModuleC = false;
				for (const m of modules) {
					if (m.identifier().endsWith("b.js")) {
						hasModuleB = true;
					}
					if (m.identifier().endsWith("c.js")) {
						hasModuleC = true;
					}
				}
				if (isFirst) {
					isFirst = false;
					expect(hasModuleB).toBe(true);
					expect(hasModuleC).toBe(true);
				} else {
					expect(hasModuleB).toBe(false);
					expect(hasModuleC).toBe(false);
				}
			});
		});
	}
};

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	plugins: [plugin]
};
