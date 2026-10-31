export const load = () => Promise.all([
	import(/* webpackChunkName: "shared" */ './shared'),
	import(/* webpackChunkName: "leaf" */ './leaf'),
]);
