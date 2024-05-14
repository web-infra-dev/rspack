const rspack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new rspack.sharing.ProvideSharedPlugin({
			provides: ["./a/index.js"]
		})
	]
};
