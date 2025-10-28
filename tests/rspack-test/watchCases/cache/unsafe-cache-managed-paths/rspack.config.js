/** @type {function(): import("@rspack/core").Configuration} */
module.exports = (env, { srcPath }) => ({
	mode: "development",
	cache: true,
	snapshot: {
		managedPaths: [/^(.+?[\\/]node_modules[\\/])/]
	}
});
