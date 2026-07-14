import { createRequire } from "node:module";

export const nestedExtraArgRequiredJoinType = typeof createRequire(
	import.meta.url,
	require("path")
)("path").join;

export const nestedExtraArgResolved = createRequire(
	import.meta.url,
	require("path")
).resolve("path");
