const path = require("path");

module.exports = {
	entry: {
		main: "./index.js"
	},
	resolve: {
		tsConfig: {
			configFile: path.resolve(__dirname, "./tsconfig.json"),
			references: "auto"
		}
	}
};
