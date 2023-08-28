const fs = require("fs");
const path = require("path");
/**
 * @type {import('webpack').Configuration}
 */
module.exports = {
	mode: "production",
	devtool: false,
	entry: {
		main: "./index.js"
	},
	target: ["web", "es5"],
	optimization: {
		minimize: false,
		concatenateModules: false
	},
	output: {
		path: "rspack-dist"
	},
	plugins: [
		{
			/**
			 *
			 * @param {import('webpack').Compiler} compiler
			 */
			apply(compiler) {
				compiler.hooks.done.tap("stats", stats => {
					const statsJson = stats.toJson({ modules: true });
					const dstPath = path.resolve(
						compiler.context,
						compiler.options.output.path,
						"stats.json"
					);
					fs.writeFileSync(dstPath, JSON.stringify(statsJson, null, 2));
				});
			}
		}
	]
};
if (process.env.RSPACK) {
	module.exports.builtins = {
		treeShaking: false
	};
}
