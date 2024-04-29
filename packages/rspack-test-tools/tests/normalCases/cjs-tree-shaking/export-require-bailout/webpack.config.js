module.exports = {
	mode: "production",
	optimization: {
		sideEffects: true
	},
	experiments: {
		rspackFuture: {
			newTreeshaking: false
		}
	}
};
