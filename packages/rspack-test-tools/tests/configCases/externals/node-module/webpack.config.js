module.exports = {
	target: "node",
	output: {
		module: true,
		chunkFormat: "module",
		filename: "[name].mjs"
	},
	experiments: {
		outputModule: true
	}
};
