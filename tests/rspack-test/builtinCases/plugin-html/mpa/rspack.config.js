const { HtmlRspackPlugin } = require("@rspack/core");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		chunk1: {
			import: ["./chunk1.js"]
		},
		chunk2: {
			import: ["./chunk2.js"]
		},
		chunk3: {
			import: ["./chunk3.js"]
		}
	},
	plugins: [
		new HtmlRspackPlugin({
			template: "index.html",
			filename: "chunk1.html",
			chunks: ["chunk1"]
		}),
		new HtmlRspackPlugin({
			template: "index.html",
			filename: "chunk2.html",
			chunks: ["chunk2"]
		}),
		new HtmlRspackPlugin({
			template: "index.html",
			filename: "chunk3.html",
			chunks: ["chunk3"]
		})
	],

};
