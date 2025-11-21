/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		polyfill: "./polyfill.js",
		main: {
			dependOn: "polyfill",
			import: "./index.js",
		},
	},
	output: {
		filename: "[name].js",
	},
	target: "web",
	optimization: {
		runtimeChunk: { name: "runtime" },
	},
	node: {
		__dirname: false,
	}
};
