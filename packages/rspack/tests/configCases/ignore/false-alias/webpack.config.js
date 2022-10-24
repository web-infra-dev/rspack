module.exports = {
	entry: "./index.js",
	resolve: {
		alias: {
			"ignored-module": false,
			"./ignored-module": false
		}
	}
};
