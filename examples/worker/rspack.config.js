const path = require("path");

module.exports = {
	entry: "./example.js",
	context: __dirname,
	output: {
		path: path.join(__dirname, "dist"),
	},
	builtins: {
		html: [
			{
				template: "./index.html"
			}
		]
	}
};
