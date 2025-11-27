const { rspack } = require("@rspack/core");
const path = require("path");
const fs = require("fs");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		index: "./index.js",
		other: "./other-entry.js"
	},
	output: {
		filename: "[name].js",
	},
	optimization: {
		runtimeChunk: 'single'
	},
	plugins: [new rspack.HtmlRspackPlugin({
		template: "./index.html",
		}),
		{
			apply(compiler) {
				compiler.hooks.done.tap("TestAssert", ()=>{
					let outputPath = compiler.options.output.path;
					const htmlContent = fs.readFileSync(path.join(outputPath,'index.html'), "utf-8");

					expect(htmlContent.match(/runtime\.js/g)).toHaveLength(1);
				})
			}
		}
	]
};
