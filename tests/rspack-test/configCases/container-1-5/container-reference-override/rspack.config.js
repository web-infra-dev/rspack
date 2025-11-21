const { ModuleFederationPlugin } = require("@rspack/core").container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new ModuleFederationPlugin({
			remoteType: "var",
			remotes: {
				abc: "ABC"
			},
			shared: {
				"./new-test": {
					shareKey: "test",
					version: false
				}
			}
		})
	]
};
