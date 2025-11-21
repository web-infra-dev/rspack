const path = require("path");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "node",
	output: {
		filename: "[name].js"
	},
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.make.tap("make", compilation => {
					const childEntry = path.resolve(__dirname, "./child-entry.js");
					const childCompiler = compilation.createChildCompiler("name", {}, [
						new compiler.webpack.EntryPlugin(compiler.context, childEntry)
					]);
					childCompiler.compile(() => {});
				});
			}
		}
	],
	optimization: {
		splitChunks: {
			cacheGroups: {
				singleVendor: {
					chunks: "all",
					enforce: true,
					name: "vendor"
				}
			}
		}
	}
};
