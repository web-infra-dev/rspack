const path = require("path");

module.exports = {
	mode: "development",
	entry: {
		main: path.resolve(__dirname, "./src/index.js")
	},
	dev: {
		port: 8081,
		static: {
			directory: path.resolve(__dirname, "../../dist")
		}
	},
	module: {
		rules: [
			{
				test: "\\.(t|j)sx?$",
				uses: [
					{
						builtinLoader: "react-refresh-runtime-loader"
					}
				]
			}
		]
	},
	builtins: {
		html: [
			{
				template: path.resolve(__dirname, "./index.html")
			}
		]
	},
	define: {
		"process.env.NODE_ENV": "development"
	}
};
