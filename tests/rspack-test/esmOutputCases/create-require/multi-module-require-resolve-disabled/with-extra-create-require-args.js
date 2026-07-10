import { createRequire } from "node:module";

let variableExtraArgRan = false;
const require = createRequire(import.meta.url, (variableExtraArgRan = true));

let inlineExtraArgRan = false;
export const extraArgRequired = require("./dep.js");
export const inlineExtraArgResolved = createRequire(
	import.meta.url,
	(inlineExtraArgRan = true)
).resolve("path");
export const extraArgEffects = [variableExtraArgRan, inlineExtraArgRan];
