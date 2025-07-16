const validateShareUsage = require("./validate-share-usage");

module.exports = {
	description:
		"ShareUsagePlugin should auto-generate usage analysis JSON with valid module IDs",
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
		// STRICT validation - must have valid share-usage.json with module IDs
		try {
			console.log("🔍 Validating ShareUsagePlugin output...");
			validateShareUsage(context.getDist());
			return Promise.resolve();
		} catch (err) {
			console.error("❌ STRICT VALIDATION FAILED:", err.message);
			return Promise.reject(err);
		}
	}
};
