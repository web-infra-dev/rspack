const path = require("path");
const { rspack } = require("@rspack/core");

const CHILD_ID = "child";
const CHILD_FILENAME = "./child";

/**
 * @type {import("@rspack/core").Configuration}
 */
module.exports = {
	devtool: "source-map",
	node: {
		__dirname: false,
		__filename: false
	},
	externals: {
		CHILD_FILENAME: `commonjs ${CHILD_FILENAME}`
	},
	output: {
		filename: "[name].js"
	},
	plugins: [
		new rspack.DefinePlugin({
			CONTEXT: JSON.stringify(__dirname)
		}),
		compiler => {
			compiler.hooks.make.tapAsync("PLUGIN", (compilation, callback) => {
				const outputOptions = {};
				const childCompiler = compilation.createChildCompiler(
					CHILD_ID,
					outputOptions
				);
				const SingleEntryPlugin = compiler.webpack.EntryPlugin;
				new SingleEntryPlugin(
					compiler.context,
					path.join(__dirname, CHILD_FILENAME),
					CHILD_ID
				).apply(childCompiler);
				childCompiler.runAsChild(err => callback(err));
			});
		}
	]
};
