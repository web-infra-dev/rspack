/** @type {import("@rspack/test-tools").TStatsAPICaseConfig} */
export default {
	description: "should report each runtime module instance's own size and chunks",
	options(context) {
		return {
			context: context.getSource(),
			mode: "development",
			devtool: false,
			entry: {
				short: "./fixtures/runtime-module-instances/short",
				long: "./fixtures/runtime-module-instances/long"
			},
			output: { chunkFilename: "[name].js" },
			optimization: {
				runtimeChunk: false,
				splitChunks: false,
				chunkIds: "deterministic",
				minimize: false
			}
		};
	},
	check(stats) {
		const json = stats.toJson({
			all: false,
			modules: true,
			chunks: true,
			ids: true,
			runtimeModules: true,
			groupModulesByType: false,
			modulesSpace: Infinity
		});
		const runtimeModules = json.modules.filter(module => module.moduleType === "runtime");
		const filenameModules = runtimeModules.filter(
			module => module.identifier === "webpack/runtime/get javascript chunk filename"
		);
		const entryChunks = json.chunks.filter(chunk => chunk.entry);
		expect(filenameModules).toHaveLength(2);
		expect(entryChunks).toHaveLength(2);
		expect(filenameModules.map(module => module.chunks[0]).sort()).toEqual(
			entryChunks.map(chunk => chunk.id).sort()
		);
		expect(new Set(filenameModules.map(module => module.size)).size).toBe(2);
		for (const chunk of entryChunks) {
			const modules = runtimeModules.filter(module => module.chunks.includes(chunk.id));
			expect(modules.reduce((size, module) => size + module.size, 0)).toBe(chunk.sizes.runtime);
		}
	}
};
