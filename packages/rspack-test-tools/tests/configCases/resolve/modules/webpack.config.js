const path = require("path");

module.exports = {
	resolve: {
		modules: [path.resolve(__dirname, "a"), path.resolve(__dirname, "b")]
	}
};
