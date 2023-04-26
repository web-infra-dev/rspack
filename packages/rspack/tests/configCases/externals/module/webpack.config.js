module.exports = {
	externals: ["path"],
	externalsType: "module",
	output: {
		module: true,
		chunkFormat: "module",
		filename: "[name].js"
	},
	experiments: {
		outputModule: true
	}
};
