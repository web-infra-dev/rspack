class Plugin {
	constructor(expectedModuleIds) {
		this.expectedModuleIds = expectedModuleIds;
	}

	apply(compiler) {
		compiler.hooks.compilation.tap("Test", compilation => {
			compilation.hooks.processAssets.tap("Test", () => {
				const moduleIds = Array.from(compilation.modules).map(m =>
					compilation.chunkGraph.getModuleId(m)
				);
				expect(moduleIds).toEqual(this.expectedModuleIds);
			});
		});
	}
}

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
	{
		optimization: {
			moduleIds: "named"
		},
		plugins: [new Plugin(["./index.js"])]
	},
	{
		optimization: {
			moduleIds: "natural"
		},
		plugins: [new Plugin([0])]
	},
	{
		optimization: {
			moduleIds: "deterministic"
		},
		plugins: [new Plugin([237])]
	}
];
