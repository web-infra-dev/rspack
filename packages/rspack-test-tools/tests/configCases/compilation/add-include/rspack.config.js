const path = require("path");
const fs = require("fs");

const PLUGIN_NAME = "plugin";

class Plugin {
	/**
	 * @param {import("@rspack/core").Compiler} compiler
	 */
	apply(compiler) {
		const { EntryPlugin, EntryDependency } = compiler.webpack;

		const modules = {};

		const fooDependency = EntryPlugin.createDependency(
			path.resolve(__dirname, "foo.js")
		);
		const barDependency = EntryPlugin.createDependency(
			path.resolve(__dirname, "bar.js")
		);

		expect(fooDependency instanceof EntryDependency).toBeTruthy();

		compiler.hooks.finishMake.tapPromise(PLUGIN_NAME, compilation => {
			const tasks = [];
			tasks.push(
				new Promise((resolve, reject) => {
					compilation.addInclude(
						compiler.context,
						fooDependency,
						{},
						(err, module) => {
							if (err) {
								reject(err);
								return;
							}
							const exportsInfo =
								compilation.moduleGraph.getExportsInfo(module);
							exportsInfo.setUsedInUnknownWay("main");
							modules["foo"] = module;
							resolve(module);
						}
					);
				})
			);
			tasks.push(
				new Promise((resolve, reject) => {
					compilation.addInclude(
						compiler.context,
						barDependency,
						{},
						(err, module) => {
							if (err) {
								reject(err);
								return;
							}
							const exportsInfo =
								compilation.moduleGraph.getExportsInfo(module);
							exportsInfo.setUsedInUnknownWay("main");
							modules["bar"] = module;
							resolve(module);
						}
					);
				})
			);
			tasks.push(
				new Promise(resolve => {
					compilation.addInclude(
						compiler.context,
						EntryPlugin.createDependency(
							path.resolve(__dirname, "no-exist.js")
						),
						{},
						(err, module) => {
							expect(err.message).toMatch(/Can't resolve/);
							resolve(module);
						}
					);
				})
			);
			return Promise.all(tasks);
		});

		const manifest = {};
		compiler.hooks.compilation.tap(PLUGIN_NAME, compilation => {
			compilation.hooks.processAssets.tap(PLUGIN_NAME, () => {
				for (const [key, module] of Object.entries(modules)) {
					const moduleId = compilation.chunkGraph.getModuleId(module);
					manifest[key] = moduleId;
				}
				fs.writeFileSync(
					path.join(compiler.outputPath, "manifest.json"),
					JSON.stringify(manifest),
					"utf-8"
				);

				const fooModule = compilation.moduleGraph.getModule(fooDependency);
				expect(path.normalize(fooModule.request)).toBe(
					path.resolve(__dirname, "./foo.js")
				);

				const barModule = compilation.moduleGraph.getModule(barDependency);
				expect(path.normalize(barModule.request)).toBe(
					path.resolve(__dirname, "./bar.js")
				);
			});
		});
	}
}

/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	entry: "./index.js",
	plugins: [new Plugin()]
};
