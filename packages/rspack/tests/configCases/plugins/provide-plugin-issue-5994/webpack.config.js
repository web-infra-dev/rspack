const { ProvidePlugin } = require("@rspack/core");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new ProvidePlugin({
			Mod: ["./harmony", "default"],
			Def: ["./harmony", "default"]
		})
	]
};
