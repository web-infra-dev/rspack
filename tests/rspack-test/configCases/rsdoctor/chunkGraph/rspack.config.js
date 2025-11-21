const {
	experiments: { RsdoctorPlugin }
} = require("@rspack/core");
const fs = require("fs");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		a: "./a.js",
		b: "./b.js"
	},
	output: {
		filename: "[name].js"
	},
	plugins: [
		new RsdoctorPlugin({
			moduleGraphFeatures: false,
			chunkGraphFeatures: ["graph"]
		}),
		{
			apply(compiler) {
				compiler.hooks.compilation.tap("TestPlugin::Chunks", compilation => {
					const hooks = RsdoctorPlugin.getCompilationHooks(compilation);
					hooks.chunkGraph.tap("TestPlugin::Chunks", data => {
						const { chunks } = data;
						expect(chunks.length).toBe(4);
						expect(chunks.filter(c => c.entry).length).toBe(2);
						expect(chunks.filter(c => c.initial).length).toBe(2);

						const entryA = chunks.find(c => c.name === "a");
						const entryB = chunks.find(c => c.name === "b");

						expect(entryA.dependencies.length).toBe(1);
						expect(entryB.dependencies.length).toBe(1);
						for (const chunk of chunks) {
							if (!chunk.name) {
								expect(chunk.imported.length).toBe(1);
							}
						}
					});
				});
			}
		},
		{
			apply(compiler) {
				compiler.hooks.compilation.tap(
					"TestPlugin::Entrypoints",
					compilation => {
						const hooks = RsdoctorPlugin.getCompilationHooks(compilation);
						hooks.chunkGraph.tap("TestPlugin::Entrypoints", data => {
							const { entrypoints } = data;
							expect(entrypoints.length).toBe(2);

							const entrypointA = entrypoints.find(e => e.name === "a");
							const entrypointB = entrypoints.find(e => e.name === "b");
							expect(entrypointA.chunks.length).toBe(1);
							expect(entrypointB.chunks.length).toBe(1);
						});
					}
				);
			}
		}
	]
};
