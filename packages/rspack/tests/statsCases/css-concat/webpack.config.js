const path = require("path");

module.exports = {
	optimization: {
		concatenateModules: true,
		minimize: false
	},
	experiments: {
		css: true,
		rspackFuture: {
			newTreeshaking: true
		}
	}
};
