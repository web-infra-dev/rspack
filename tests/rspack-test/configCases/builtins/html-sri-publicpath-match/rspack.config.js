const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	target: "web",
	output: {
		publicPath: "https://cdn.example.com/assets/",
		crossOriginLoading: "anonymous"
	},
	node: {
		__dirname: false
	},
	plugins: [
		new rspack.HtmlRspackPlugin({
			filename: "index.html"
		}),
		new rspack.SubresourceIntegrityPlugin({
			hashFuncNames: ["sha384"],
			enabled: true
		})
	]
};
