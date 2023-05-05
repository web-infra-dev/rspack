/** @type {import("../../../../").Configuration} */
module.exports = {
	entry: {
		main: "./index",
		misc: "./second"
	},
	output: {
		filename: "[name].js"
	},
	experiments: {
		newSplitChunks: true
	},
	optimization: {
		splitChunks: {
			minSize: 0
		}
	}
};
