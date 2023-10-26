const rspack = require("@rspack/core");
/** @type {import('@rspack/cli').Configuration} */
const config = {
	entry: {
		main: "./src/index.tsx"
	},
	plugins: [
		new rspack.HtmlRspackPlugin({
			template: "./index.html"
		})
	]
};
module.exports = config;
