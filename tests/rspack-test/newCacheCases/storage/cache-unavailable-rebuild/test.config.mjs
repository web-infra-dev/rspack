export default {
	moduleScope(scope, stats, options) {
		scope.cacheUnavailable = options.plugins.find(
			plugin => plugin.name === "UnavailableCacheTest"
		).unavailable;
		return scope;
	}
};
