class Plugin {
	/**
	 * @param {import("@rspack/core").Compiler} compiler
	 */
	apply(compiler) {
		const { ExternalModule } = compiler.webpack;

		compiler.hooks.afterEmit.tap("AutoExternalPlugin", compilation => {
			const externalModules = Array.from(compilation.modules).filter(module => module instanceof ExternalModule);

			expect(externalModules.length).toBe(1);

			const module = externalModules[0];

			expect(module.constructor.name).toBe("ExternalModule");

			expect(module.userRequest).toBe("external");

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
	entry: "./index.js",
	output: {
		library: {
			type: "commonjs"
		}
	},
	externals: {
		external: "external"
	},
	plugins: [new Plugin()]
};
