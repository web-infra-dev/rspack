class Plugin {
	apply(compiler) {
		compiler.hooks.afterEmit.tap("AutoExternalPlugin", compilation => {
			const externalModules = Array.from(compilation.modules).filter(module =>
				module.identifier().startsWith("external ")
			);

			expect(externalModules.length).toBe(1);

			externalModules.forEach(module => {
				expect(module.userRequest).toBe("external");
			});
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
