const rspack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
const basic = {
	output: {
		filename: `[name].js`,
		chunkFilename: `async.js`,
		module: true,
		library: {
			type: "modern-module"
		},
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
	plugins: [
		new rspack.experiments.RslibPlugin()
	],
	optimization: {
		concatenateModules: true,
		avoidEntryIife: true,
		minimize: false
	}
};

module.exports = [
	{
		entry: {
			main: "./main.js"
		},
		...basic
	},
	{
		entry: {
			main2: "./main2.js"
		},
		...basic
	},
	{
		entry: {
			index: "./index.js"
		},
		output: {
			module: true,
			filename: "index.mjs"
		},
		experiments: {
			outputModule: true
		}
	}
];
