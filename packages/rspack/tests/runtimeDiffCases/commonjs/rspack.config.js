module.exports = {
	optimization: {
		usedExports: true,
		providedExports: true
	},
	experiments: {
		rspackFuture: {
			newTreeshaking: true
		}
	}
};
