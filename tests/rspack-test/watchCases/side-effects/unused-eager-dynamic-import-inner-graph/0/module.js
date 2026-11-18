export const loadEagerFeature = () =>
	import(/* webpackMode: "eager" */ "./eager-feature");

export const live = "live";
