const {
	experiments: { RsdoctorPlugin }
} = require("@rspack/core");
const path = require("path");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	optimization: {
		concatenateModules: true,
		sideEffects: true
	},
	plugins: [
		new RsdoctorPlugin({
			moduleGraphFeatures: ["graph"],
			chunkGraphFeatures: false
		}),
		{
			apply(compiler) {
				compiler.hooks.compilation.tap(
					"TestPlugin::SideEffects",
					compilation => {
						const hooks = RsdoctorPlugin.getCompilationHooks(compilation);
						hooks.moduleGraph.tap(
							"TestPlugin::SideEffects",
							moduleGraph => {
								const { modules } = moduleGraph;

								// Find modules with sideEffectsLocations
								const modulesWithSideEffects = modules.filter(
									m => m.sideEffectsLocations && m.sideEffectsLocations.length > 0
								);

								expect(modulesWithSideEffects.length).toBeGreaterThan(0);

								for (const mod of modulesWithSideEffects) {
									for (const loc of mod.sideEffectsLocations) {
										// Each location should have structured data
										expect(typeof loc.location).toBe("string");
										expect(loc.location).toMatch(/^\d+:\d+(?:-(?:\d+|\d+:\d+))?$/);
										expect(typeof loc.nodeType).toBe("string");
										expect(loc.nodeType.length).toBeGreaterThan(0);
										expect(typeof loc.request).toBe("string");
									}
								}

								// Find the utils module specifically — it has console.log (a side effect)
								const utilsModule = modulesWithSideEffects.find(m =>
									m.path && m.path.includes("utils.js")
								);
								expect(utilsModule).toBeTruthy();
								expect(utilsModule.sideEffectsLocations.length).toBe(1);

								const utilsLoc = utilsModule.sideEffectsLocations[0];
								expect(utilsLoc.nodeType).toBe("Statement");

								// pure.js should NOT have side effects locations
								const pureModule = modules.find(m =>
									m.path && m.path.includes("pure.js")
								);
								if (pureModule) {
									expect(
										pureModule.sideEffectsLocations?.length ?? 0
									).toBe(0);
								}
							}
						);
					}
				);
			}
		}
	]
};