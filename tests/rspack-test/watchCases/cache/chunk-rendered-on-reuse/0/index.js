export const loadShared = () =>
	import(/* webpackChunkName: "shared" */ "./shared");

export const version = "step-0";
