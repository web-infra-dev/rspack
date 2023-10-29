const rspack = require("@rspack/core");
/** @type {import('@rspack/cli').Configuration} */
const config = {
	entry: {
		main: {
			import: ["./index.js"]
		}
	},
	plugins: [
		new rspack.HtmlRspackPlugin({
			template: "index.html"
		})
	]
};
module.exports = config;
