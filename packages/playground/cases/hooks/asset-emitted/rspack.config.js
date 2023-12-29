const rspack = require("@rspack/core");

module.exports = {
	entry: {
		main: "./src/index.js"
	},
	output: { clean: true },
	plugins: [new rspack.HtmlRspackPlugin({ template: "./src/index.html" })]
};
