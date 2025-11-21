/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	plugins: [
		{
			apply(compiler) {
				let isFirst = true;
				compiler.hooks.done.tap("PLUGIN", stats => {
					let { modules } = stats.toJson({ modules: true });
					let hasModuleA1 = false;
					let hasModuleA2 = false;
					let hasModuleB = false;
					for (const m of modules) {
						if (m.identifier.endsWith("a1.js")) {
							hasModuleA1 = true;
						}
						if (m.identifier.endsWith("a2.js")) {
							hasModuleA2 = true;
						}
						if (m.identifier.endsWith("b.js")) {
							hasModuleB = true;
						}
					}
					if (isFirst) {
						expect(hasModuleA1).toBe(true);
						expect(hasModuleA2).toBe(true);
						expect(hasModuleB).toBe(true);
					} else {
						expect(hasModuleA1).toBe(false);
						expect(hasModuleA2).toBe(false);
						expect(hasModuleB).toBe(false);
					}
					isFirst = false;
				});
			}
		}
	]
};
