const {
	experiments: { RsdoctorPlugin }
} = require("@rspack/core");
const path = require("path");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		concatenateModules: true
	},
	plugins: [
		new RsdoctorPlugin({
			moduleGraphFeatures: ["graph"],
			chunkGraphFeatures: false
		}),
		{
			apply(compiler) {
				compiler.hooks.compilation.tap("TestPlugin::Modules", compilation => {
					const hooks = RsdoctorPlugin.getCompilationHooks(compilation);
					hooks.moduleGraph.tap("TestPlugin::Modules", moduleGraph => {
						const { modules } = moduleGraph;
						expect(modules.length).toBe(5);

						const concatenateModules = modules.filter(
							module => module.kind === "concatenated"
						);
						const normalModules = modules.filter(
							module => module.kind === "normal"
						);
						expect(concatenateModules.length).toBe(1);
						expect(normalModules.length).toBe(4);

						const entryModule = modules.find(
							module => module.isEntry && module.kind === "concatenated"
						);
						expect(entryModule.chunks.length).toBe(1);
						expect(entryModule.modules.length).toBe(3);
						expect(entryModule.dependencies.length).toBe(1);
						expect(entryModule.path).toBe(path.join(__dirname, "./index.js"));
					});
				});
			}
		},
		{
			apply(compiler) {
				compiler.hooks.compilation.tap(
					"TestPlugin::Dependencies",
					compilation => {
						const hooks = RsdoctorPlugin.getCompilationHooks(compilation);
						hooks.moduleGraph.tap("TestPlugin::Dependencies", moduleGraph => {
							const { dependencies } = moduleGraph;
							const deps = dependencies.map(dep => ({
								request: dep.request,
								kind: dep.kind
							}));
							deps.sort((a, b) => (a.request > b.request ? 1 : -1));
							expect(deps).toEqual([
								{ request: "./b", kind: "esm export" },
								{ request: "./c", kind: "esm export" },
								{ request: "./lib/a", kind: "esm import" }
							]);
						});
					}
				);
			}
		},
		{
			apply(compiler) {
				compiler.hooks.compilation.tap(
					"TestPlugin::ChunkModules",
					compilation => {
						const hooks = RsdoctorPlugin.getCompilationHooks(compilation);
						hooks.moduleGraph.tap("TestPlugin::ChunkModules", moduleGraph => {
							const { chunkModules } = moduleGraph;
							expect(chunkModules.length).toBe(1);
							expect(chunkModules[0].modules.length).toBe(5);
						});
					}
				);
			}
		}
	]
};
