/** @type {import("../../../../").Configuration} */
module.exports = {
	entry: {
		main: "./index",
		second: "./index"
	},
	target: "web",
	output: {
		filename: "[name].js"
	},
	experiments: {
		newSplitChunks: true
	},
	optimization: {
		splitChunks: {
			minSize: 1
		}
	}
};
