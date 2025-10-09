/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		bundle: "./index.js",
		entry: { import: "./entry.js", runtime: "runtime" }
	},
	mode: "production",
	/// DIFF: rspack uses cache: true to enable memory cache
	// cache: {
	// 	type: "memory"
	// },
	cache: true,
	output: {
		filename: "[name].js",
		pathinfo: true,
		publicPath: "./",
		library: {
			name: ["RESULT", "value"],
			type: "assign"
		}
	},
	optimization: {
		splitChunks: {
			minSize: 1,
			chunks: "all",
			usedExports: false
		},
		minimize: false,
		concatenateModules: false
	},
	externalsType: "commonjs",
	externals: ["fs", "path"],
	node: {
		__dirname: false
	},
	target: "web"
};
