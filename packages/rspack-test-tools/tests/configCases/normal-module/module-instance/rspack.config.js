const path = require("path");

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
			expect(module.userRequest).toBe(
				`${path.join(__dirname, "passthrough-loader.js")}!${path.join(__dirname, "index.js")}`
			);
			expect(module.rawRequest).toContain("index.js");
			expect(module.resourceResolveData.fragment).toBe("");
			expect(module.resourceResolveData.path).toBe(
				path.join(__dirname, "index.js")
			);
			expect(module.resourceResolveData.query).toBe("");
			expect(module.resourceResolveData.resource).toBe(
				path.join(__dirname, "index.js")
			);
			expect(module.loaders.length).toBe(1);
			expect(
				module.loaders.map(({ loader }) =>
					path.relative(compiler.context, loader)
				)
			).toEqual(["passthrough-loader.js"]);
			expect(module.type).toBe("javascript/auto");

			expect(Object.hasOwn(module, "type")).toBe(true);
			expect(Object.hasOwn(module, "resource")).toBe(true);
			expect(Object.hasOwn(module, "userRequest")).toBe(true);
			expect(Object.hasOwn(module, "rawRequest")).toBe(true);
			expect(Object.hasOwn(module, "resourceResolveData")).toBe(true);
			expect(Object.hasOwn(module, "loaders")).toBe(true);
			expect(Object.hasOwn(module, "useSourceMap")).toBe(true);
			expect(Object.hasOwn(module, "useSimpleSourceMap")).toBe(true);
			expect(Object.hasOwn(module, "matchResource")).toBe(true);
			expect(Object.hasOwn(module, "layer")).toBe(true);
			expect(Object.hasOwn(module, "factoryMeta")).toBe(true);
			expect(Object.hasOwn(module, "buildMeta")).toBe(true);
			expect(Object.hasOwn(module, "buildMeta")).toBe(true);
		});
	}
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./passthrough-loader.js!./index.js",
	plugins: [new Plugin()]
};
