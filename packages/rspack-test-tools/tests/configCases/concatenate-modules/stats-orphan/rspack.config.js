module.exports = {
	mode: "development",
	entry: {
		main: "./index.js"
	},
	optimization: {
		concatenateModules: true,
		minimize: false
	},
	stats: {
		modules: true
	}
};
