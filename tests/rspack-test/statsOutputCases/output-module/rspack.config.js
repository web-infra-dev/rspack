module.exports = {
	mode: "production",
	entry: "./index",
	experiments: {
		outputModule: true
	},
	stats: {
		assets: true,
		modules: true,
	}
};
