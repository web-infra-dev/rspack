const rspack = require("@rspack/core");

module.exports = {
	mode: "production",
	target: "web",
	entry: {
		main: "./index.js"
	},
	output: {
		crossOriginLoading: "anonymous"
	},
	plugins: [
		new rspack.HtmlRspackPlugin(),
		new rspack.SubresourceIntegrityPlugin()
	]
};
