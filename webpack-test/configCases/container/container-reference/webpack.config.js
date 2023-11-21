const { ContainerReferencePlugin } = require("../../../../packages/rspack").container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new ContainerReferencePlugin({
			remoteType: "var",
			remotes: {
				abc: "ABC",
				def: "DEF"
			}
		})
	]
};
