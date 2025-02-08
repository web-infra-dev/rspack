/** @type {import("@rspack/core").Configuration} */
module.exports = [
	{
		entry: {
			main: "./main.js"
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
		externals: {
			react: "react-alias",
			vue: "vue-alias",
			angular: "angular-alias",
			svelte: "svelte-alias",
			lit: "lit-alias",
			solid: "solid-alias",
			jquery: "jquery-alias"
		},
		externalsType: "module-import",
		experiments: {
			outputModule: true
		},
		optimization: {
			concatenateModules: true,
			avoidEntryIife: true,
			minimize: false
		}
	},
	{
		entry: {
			index: "./index.js"
		},
		output: {
			filename: "index.js"
		}
	}
];
