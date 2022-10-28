const path = require("path");

module.exports = {
	entry: {
		main: "./index.js"
	},
	resolve: {
		tsconfigPath: path.resolve(__dirname, "./tsconfig.json")
	}
};
