const path = require("path");

const PLUGIN_NAME = "AddEntryWithImportModulePlugin";

class AddEntryWithImportModulePlugin {
	/**
	 * @param {import("@rspack/core").Compiler} compiler
	 */
	apply(compiler) {
		const { EntryPlugin } = compiler.rspack;

		compiler.hooks.make.tapPromise(PLUGIN_NAME, compilation => {
			return new Promise((resolve, reject) => {
				compilation.addEntry(
					compiler.context,
					EntryPlugin.createDependency(
						path.resolve(__dirname, "entry.js")
					),
					{ name: "dynamic" },
					(err) => {
						if (err) reject(err);
						else resolve();
					}
				);
			});
		});
	}
}

/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	output: {
		filename: "[name].js"
	},
	module: {
		rules: [
			{
				test: /entry\.js$/,
				use: [{ loader: "./loader", options: {} }],
				type: "asset/source"
			}
		]
	},
	plugins: [new AddEntryWithImportModulePlugin()]
};
