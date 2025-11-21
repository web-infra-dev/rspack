const { ProvidePlugin } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new ProvidePlugin({
			process: ["./process.js"],
			name: ["./name.js"]
		})
	],
};
