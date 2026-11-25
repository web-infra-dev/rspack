import { label } from "./shared.js";

export const loadVendor = () =>
	import(/* webpackChunkName: "vendor-lazy" */ "./vendor-lazy.js").then(
		({ default: value }) => `${label}: ${value}`
	);
