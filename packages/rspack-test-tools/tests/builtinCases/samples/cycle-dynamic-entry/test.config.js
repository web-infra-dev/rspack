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
