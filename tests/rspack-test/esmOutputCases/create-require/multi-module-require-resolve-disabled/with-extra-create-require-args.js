import { createRequire } from "node:module";

let variableExtraArgRan = false;
const require = createRequire(import.meta.url, (variableExtraArgRan = true));

let inlineExtraArgRan = false;
let inlineRequireExtraArgRan = false;
let inlineCacheExtraArgRan = false;
export const extraArgRequired = require("./dep.js");
export const inlineExtraArgRequired = createRequire(
	import.meta.url,
	(inlineRequireExtraArgRan = true),
	require("path")
)("./dep.js");
export const inlineExtraArgResolved = createRequire(
	import.meta.url,
	(inlineExtraArgRan = true)
).resolve("path");
export const inlineExtraArgCacheType = typeof createRequire(
	import.meta.url,
	(inlineCacheExtraArgRan = true),
	require("path")
).cache;
export const extraArgEffects = [
	variableExtraArgRan,
	inlineExtraArgRan,
	inlineRequireExtraArgRan,
	inlineCacheExtraArgRan
];
