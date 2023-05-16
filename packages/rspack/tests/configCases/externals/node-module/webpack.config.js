module.exports = {
	target: "node",
	output: {
		module: true,
		chunkFormat: "module",
		filename: "[name].js"
	},
	experiments: {
		outputModule: true
	}
};
