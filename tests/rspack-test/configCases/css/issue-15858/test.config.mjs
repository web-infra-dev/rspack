export default {
	findBundle(index, options) {
		const name = options.name;
		const runtime = name.startsWith("single") ? "runtime" : "runtime~main";
		// Never execute the other entry. The preloaded variants check that a
		// stylesheet already supplied by the page is reused.
		return [
			...(name.endsWith("-preloaded") ? [`shared-css.${name}.css`] : []),
			`${runtime}.${name}.js`,
			`main.${name}.js`
		];
	},
	moduleScope(scope, stats, options) {
		scope.runtimeChunkMode = options.name;
		scope.preloadedCss = options.name.endsWith("-preloaded");
	}
};
