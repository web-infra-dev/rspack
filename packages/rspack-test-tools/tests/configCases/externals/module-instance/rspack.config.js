class Plugin {
	/**
	 * @param {import("@rspack/core").Compiler} compiler
	 */
	apply(compiler) {
		const { ExternalModule } = compiler.webpack;

		compiler.hooks.afterEmit.tap("AutoExternalPlugin", compilation => {
			const externalModules = Array.from(compilation.modules).filter(
				module => module instanceof ExternalModule
			);

			expect(externalModules.length).toBe(1);

			const module = externalModules[0];

			expect(module.constructor.name).toBe("ExternalModule");
			expect(module.type).toBe("javascript/dynamic");
			expect(module.userRequest).toBe("external");
			expect(module.useSourceMap).toBe(false);
			expect(module.useSimpleSourceMap).toBe(false);

			expect(Object.hasOwn(module, "type")).toBe(true);
			expect(Object.hasOwn(module, "context")).toBe(true);
			expect(Object.hasOwn(module, "layer")).toBe(true);
			expect(Object.hasOwn(module, "userRequest")).toBe(true);
			expect(Object.hasOwn(module, "factoryMeta")).toBe(true);
			expect(Object.hasOwn(module, "useSourceMap")).toBe(true);
			expect(Object.hasOwn(module, "useSimpleSourceMap")).toBe(true);
			expect(Object.hasOwn(module, "buildMeta")).toBe(true);
			expect(Object.hasOwn(module, "buildMeta")).toBe(true);
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
