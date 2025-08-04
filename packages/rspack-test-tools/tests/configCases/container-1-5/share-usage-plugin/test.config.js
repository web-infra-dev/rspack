const validateShareUsage = require("./validate-share-usage");

module.exports = {
	description:
		"ShareUsagePlugin should accurately track used/unused exports for CJS, ESM, and local shared modules",
	options(context) {
		return {
			experiments: {
				outputModule: false,
				rspackFuture: {
					bundlerInfo: {
						force: false
					}
				}
			},
			cache: false // Disable cache to ensure fresh analysis
		};
	},
	diffStats: true,
	nonEsmThis: "(global || {})",
	afterBuild(context) {
		// Run strict validation with proper assertions
		try {
			validateShareUsage(context.getDist());
			return Promise.resolve();
		} catch (err) {
			return Promise.reject(err);
		}
	}
};
