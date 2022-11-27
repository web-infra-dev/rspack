const path = require("path");

module.exports = {
	entry: {
		main: "./index.js"
	},
	resolve: {
		tsConfigPath: path.resolve(__dirname, "./tsconfig.json")
	}
};
