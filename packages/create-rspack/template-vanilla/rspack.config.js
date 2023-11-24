const rspack = require("@rspack/core");
module.exports = {
	entry: {
		main: "./src/index.js"
	},
	plugins: [new rspack.HtmlRspackPlugin({ template: "./index.html" })]
};
