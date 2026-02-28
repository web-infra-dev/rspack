const { ModuleFederationPlugin } = require("@rspack/core").container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./src/index.js",
	plugins: [
		new ModuleFederationPlugin({
			shared: ["./src/shared.js"]
		})
	]
};
