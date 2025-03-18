class Plugin {
	/**
	 * @param {import("@rspack/core").Compiler} compiler
	 */
	apply(compiler) {
		const { NormalModule } = compiler.webpack;

		compiler.hooks.afterEmit.tap("AutoExternalPlugin", compilation => {
			const normalModules = Array.from(compilation.modules).filter(
				module => module instanceof NormalModule
			);

			expect(normalModules.length).toBe(1);

			const module = normalModules[0];

			expect(module instanceof NormalModule).toBe(true);
			expect(module.constructor.name).toBe("NormalModule");

			expect(module.resource).toContain("index.js");
			expect(module.userRequest).toContain("index.js");
			expect(module.rawRequest).toContain("index.js");
			expect(module.resourceResolveData.fragment).toBe("");
			expect(module.resourceResolveData.path).toContain("index.js");
			expect(module.resourceResolveData.query).toBe("");
			expect(module.resourceResolveData.resource).toContain("index.js");
			expect("matchResource" in module).toBe(true);
			expect(module.loaders.length).toBe(1);

			expect(module.type).toBe("javascript/auto");
			expect("context" in module).toBe(true);
			expect("layer" in module).toBe(true);
			expect("factoryMeta" in module).toBe(true);
			expect(module.useSourceMap).toBe(false);
			expect(module.useSimpleSourceMap).toBe(false);
			expect("buildMeta" in module).toBe(true);
			expect("buildMeta" in module).toBe(true);
		});
	}
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./passthrough-loader.js!./index.js",
	plugins: [new Plugin()]
};
