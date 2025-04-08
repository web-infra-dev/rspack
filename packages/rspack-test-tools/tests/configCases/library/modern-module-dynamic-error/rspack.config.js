/** @type {import("@rspack/core").Configuration} */

module.exports = {
	entry: {
		index: "./index.js"
	},
	output: {
		filename: `[name].js`,
		chunkFilename: `async.js`,
		module: true,
		library: {
			type: "modern-module"
		},
		iife: false,
		chunkFormat: "module",
		chunkLoading: "import"
	},
	externalsType: "module-import",
	experiments: {
		outputModule: true
	},
	optimization: {
		concatenateModules: true,
		avoidEntryIife: true,
		minimize: false
	},
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.done.tap("TestPlugin", stats => {
					const errors = stats.toJson({
						errors: true,
						ids: true,
						moduleTrace: true
					}).errors;
					expect(errors.length).toBe(1);
					expect(errors[0].message).toMatch(
						/Module not found: Can't resolve 'non_exist_dep' in/
					);
				});
			}
		}
	]
};
