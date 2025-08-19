const validateShareUsage = require("./validate-share-usage");

module.exports = {
	findBundle(i, options) {
		return ["main.js"];
	},
	timeout: 30000,
	afterBuild(context) {
		// Validate inter-shared-module dependency tracking
		try {
			validateShareUsage(context.getDist());
			return Promise.resolve();
		} catch (err) {
			return Promise.reject(err);
		}
	}
};
