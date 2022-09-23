const path = require("path");

module.exports = {
	mode: "development",
	entry: {
		main: path.resolve(__dirname, "./src/index.js")
	},
	dev: {
		port: 8081,
		static: {
			directory: "dist"
		}
	},
	define: {
		"process.env.NODE_ENV": "development"
	},
	builtins: {
		html: [
			{
				template: path.resolve(__dirname, "./index.html")
			}
		]
	}
};
