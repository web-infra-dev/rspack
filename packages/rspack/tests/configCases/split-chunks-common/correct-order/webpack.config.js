/** @type {import("../../../../").Configuration} */
module.exports = {
	entry: {
		vendor: ["./a"],
		main: "./index"
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
			minSize: 1,
			name: "vendor"
		}
	}
};
