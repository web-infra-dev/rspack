const { SplitChunksPlugin } = require("@rspack/core").optimize;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		vendor: ["./a"],
		main: "./index"
	},
	target: "web",
	output: {
		filename: "[name].js"
	},
	optimization: {
		splitChunks: false
	},
	plugins: [new SplitChunksPlugin()]
};
