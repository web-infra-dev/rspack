const path = require("path");
const { DefinePlugin, ProvidePlugin } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		concatenateModules: true
	},
	plugins: [
		new DefinePlugin({
			"process.env.INLINED_VALUE": JSON.stringify("inlined")
		}),
		new ProvidePlugin({
			process: path.resolve(__dirname, "process.js")
		})
	]
};
