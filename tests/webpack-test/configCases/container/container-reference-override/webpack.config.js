const { ContainerReferencePlugin } = require("@rspack/core").container;
const { ProvideSharedPlugin } = require("@rspack/core").sharing;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new ContainerReferencePlugin({
			remoteType: "var",
			remotes: {
				abc: "ABC"
			}
		}),
		new ProvideSharedPlugin({
			provides: {
				"./new-test": {
					shareKey: "test",
					version: false
				}
			}
		})
	]
};
