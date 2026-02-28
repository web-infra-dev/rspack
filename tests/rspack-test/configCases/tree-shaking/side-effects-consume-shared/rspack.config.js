const { rspack } = require("@rspack/core");

/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	optimization: {
		sideEffects: true
	},
	plugins: [
		new rspack.sharing.ConsumeSharedPlugin({
			consumes: {
				"./lib/c.js": {
					singleton: true,
					eager: true
				}
			}
		})
	]
};
