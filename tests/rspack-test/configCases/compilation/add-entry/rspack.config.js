const path = require("path");

const PLUGIN_NAME = "Plugin";

class Plugin {
	/**
	 * @param {import("@rspack/core").Compiler} compiler
	 */
	apply(compiler) {
		const { EntryPlugin } = compiler.webpack;

		const fooDependency = EntryPlugin.createDependency(
			path.resolve(__dirname, "foo.js")
		);
		const barDependency = EntryPlugin.createDependency(
			path.resolve(__dirname, "bar.js")
		);

		const modules = {};

		compiler.hooks.make.tapPromise(PLUGIN_NAME, compilation => {
			const tasks = [];
			tasks.push(
				new Promise((resolve, reject) => {
					compilation.addEntry(
						compiler.context,
						fooDependency,
						"foo",
						(err, module) => {
							if (err) {
								reject(err);
								return;
							}
							modules.foo = module;
							resolve();
						}
					);
				})
			);
			tasks.push(
				new Promise((resolve, reject) => {
					compilation.addEntry(
						compiler.context,
						barDependency,
						{
							name: "bar"
						},
						(err, module) => {
							if (err) {
								reject(err);
								return;
							}
							modules.bar = module;
							resolve();
						}
					);
				})
			);
			return Promise.all(tasks);
		});

		compiler.hooks.compilation.tap(PLUGIN_NAME, compilation => {
			compilation.hooks.processAssets.tap(PLUGIN_NAME, () => {
				const fooModule = compilation.moduleGraph.getModule(fooDependency);
				expect(fooModule).toBe(modules.foo);

				const barModule = compilation.moduleGraph.getModule(barDependency);
				expect(barModule).toBe(modules.bar);
			});
		});
	}
}

/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	output: {
		filename: "[name].js"
	},
	plugins: [new Plugin()]
};
