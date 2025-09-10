const plugin = {
	name: "PLUGIN",
	apply(compiler) {
		let isFirst = true;
		compiler.hooks.compilation.tap("PLUGIN", compilation => {
			compilation.hooks.finishModules.tap("PLUGIN", modules => {
				let hasModuleA = false;
				let hasModuleB = false;
				let hasModuleC = false;
				let hasModuleD = false;
				for (const m of modules) {
					if (m.identifier().endsWith("a.js")) {
						hasModuleA = true;
					}
					if (m.identifier().endsWith("b.js")) {
						hasModuleB = true;
					}
					if (m.identifier().endsWith("c.js")) {
						hasModuleC = true;
					}
					if (m.identifier().endsWith("d.js")) {
						hasModuleD = true;
					}
				}
				if (isFirst) {
					isFirst = false;
					expect(hasModuleA && hasModuleB && hasModuleC && hasModuleD).toBe(
						true
					);
				} else {
					expect(hasModuleA || hasModuleB || hasModuleC || hasModuleD).toBe(
						false
					);
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
