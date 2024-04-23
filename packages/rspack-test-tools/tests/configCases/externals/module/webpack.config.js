module.exports = {
	externals: ["path"],
	externalsType: "module",
	output: {
		module: true,
		chunkFormat: "module",
		filename: "[name].mjs"
	},
	experiments: {
		outputModule: true
	}
};
