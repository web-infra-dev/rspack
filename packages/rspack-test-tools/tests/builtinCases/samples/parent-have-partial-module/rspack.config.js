/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		index: {
			import: ["./index.js"]
		}
	},
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
