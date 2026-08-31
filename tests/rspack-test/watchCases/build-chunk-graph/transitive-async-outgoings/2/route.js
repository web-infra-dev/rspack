export { value } from "./leaf";

export const load = () =>
	import(
		/* webpackChunkName: "next", webpackPrefetch: 1 */ "./lazy"
	);

export const loadSecond = () =>
	import(
		/* webpackChunkName: "lazy", webpackPrefetch: 2 */ "./lazy-second"
	);
