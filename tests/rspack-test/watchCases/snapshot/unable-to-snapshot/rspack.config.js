const path = require("path");
/** @type {function(any, any): import("@rspack/core").Configuration} */
module.exports = (env, { tempPath }) => ({
	cache: {
		type: "memory"
	},
	snapshot: {
		managedPaths: [path.resolve(tempPath, "node_modules")]
	},
	module: {
		unsafeCache: false
	}
});
