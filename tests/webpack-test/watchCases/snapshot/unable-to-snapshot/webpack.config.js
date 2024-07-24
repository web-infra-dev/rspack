const path = require("path");
/** @type {function(any, any): import("@rspack/core").Configuration} */
module.exports = (env, { srcPath }) => ({
	cache: {
		type: "memory"
	},
	snapshot: {
		managedPaths: [path.resolve(srcPath, "node_modules")]
	},
	module: {
		unsafeCache: false
	}
});
