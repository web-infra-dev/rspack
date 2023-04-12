const path = require("path");

module.exports = {
	target: "web",
	externals: {
		path: "require('path')",
		fs: "require('fs')"
	},
	builtins: {
		define: {
			__dirname: JSON.stringify(path.join(__dirname, "./dist"))
		},
		html: [
			{
				filename: "main_page/index.html"
			}
		]
	}
};
