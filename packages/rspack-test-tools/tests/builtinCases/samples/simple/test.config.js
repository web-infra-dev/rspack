/** @type {import("../../../../../rspack").Configuration} */
module.exports = {
	optimization: {
		removeAvailableModules: true,
		providedExports: true,
		usedExports: "global"
	},
	experiments: {
		rspackFuture: {
			newTreeshaking: true
		}
	}
};
