export default {
	moduleScope(scope, _stats, options) {
		scope.sharedFallbacks = options.plugins.find(
			plugin => plugin.name === 'TreeShakingSharedPlugin'
		).buildAssets;
	}
};
