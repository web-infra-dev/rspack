const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	target: "web",
	entry: "./main.js",
	output: {
		filename: "main.js",
		publicPath: "https://cdn.example.com/assets/",
		crossOriginLoading: "anonymous"
	},
	plugins: [
		new rspack.HtmlRspackPlugin(),
		new rspack.SubresourceIntegrityPlugin({
			hashFuncNames: ["sha384"],
			enabled: true
		})
	]
};
