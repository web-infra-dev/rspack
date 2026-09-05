export const feature = () =>
	import(/* webpackChunkName: "feature" */ "./feature");

export const always = () =>
	import(/* webpackChunkName: "always" */ "./always");

export const live = "live";
