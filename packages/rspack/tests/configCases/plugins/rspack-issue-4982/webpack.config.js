const rspack = require("@rspack/core");
const path = require("path");
const assert = require("assert");

module.exports = {
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.make.tap("child", compilation => {
					const childCompiler = compilation.createChildCompiler(
						"child",
						{
							filename: "child.js"
						},
						[
							new compiler.webpack.EntryPlugin(
								compiler.context,
								path.resolve(__dirname, "./child.js"),
								{ name: "child" }
							)
						]
					);
					childCompiler.compile((_err, result) => {
						const assets = result
							.getAssets()
							.filter(asset => asset.name === "child.js");
						assert(assets.length === 1);
						const asset = assets[0];
						assert(asset.source.source().toString().includes("hello/1"));
					});
				});
			}
		}
	],
	optimization: {
		minimize: true,
		minimizer: [new rspack.SwcJsMinimizerRspackPlugin()]
	}
};
