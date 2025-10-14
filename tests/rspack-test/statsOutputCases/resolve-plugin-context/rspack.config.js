var ResolvePackageFromRootPlugin = require("./ResolvePackageFromRootPlugin");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	entry: "./index",
	output: {
		filename: "bundle.js"
	},
	resolve: {
		plugins: [new ResolvePackageFromRootPlugin(__dirname)]
	},
	stats: {
		chunkModules: false,
		modules: true
	}
};
