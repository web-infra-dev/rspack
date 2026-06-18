import { createRequire } from "node:module";

// A CUSTOM createRequire source: rspack must keep calls to `makeRequire` instead of replacing
// them with Node's built-in `__rspack_createRequire`. The marker distinguishes the two at
// runtime: Node's createRequire result would not have it.
export function makeRequire(url) {
	const require = createRequire(url);
	require.__fromShim = true;
	return require;
}
