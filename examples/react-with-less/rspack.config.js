const path = require("path");
module.exports = {
	context: __dirname,
	mode: "development",
	entry: {
		main: ["./src/index.jsx"]
	},
	builtins: {
		html: [{}],
		define: {
			"process.env.NODE_ENV": "'development'"
		}
	},
	module: {
		rules: [
			{
				test: /.less$/,
				use: ["less-loader"],
				type: "css"
			}
		]
	},
	output: {
		path: path.resolve(__dirname, "dist")
	}
};
