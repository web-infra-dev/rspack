const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	entry: "./main.js",
	output: {
		filename: "main.js"
	},
	plugins: [
		new rspack.HtmlRspackPlugin({
			tags: [
				{
					tag: "script",
					attrs: {
						src: "https://cdn.jsdelivr.net/npm/react@18/umd/react.production.min.js"
					},
					append: false
				}
			]
		}),
		new rspack.SubresourceIntegrityPlugin({
			hashFuncNames: ["sha384"],
			enabled: true
		})
	]
};
