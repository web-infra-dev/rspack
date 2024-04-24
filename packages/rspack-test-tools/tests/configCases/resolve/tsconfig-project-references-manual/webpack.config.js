const path = require("path");

module.exports = {
	entry: {
		main: "./index.js"
	},
	resolve: {
		tsConfig: {
			configFile: path.resolve(__dirname, "./tsconfig.json"),
			references: [
				path.resolve(__dirname, "./project_a/conf.json"),
				path.resolve(__dirname, "./project_b"),
				path.resolve(__dirname, "./project_c/tsconfig.json")
			]
		}
	}
};
