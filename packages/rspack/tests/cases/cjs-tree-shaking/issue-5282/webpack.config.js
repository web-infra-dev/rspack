module.exports = {
	mode: "production",
	optimization: {
		minimize: false,
		moduleIds: "named"
	},
	experiments: {
		rspackFuture: {
			newTreeshaking: true
		}
	}
};
