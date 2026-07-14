import { createRequire } from "node:module";

let directRequire = createRequire(import.meta.url);
const directReplacement = createRequire(import.meta.url);
globalThis.__RSPACK_USE_REPLACEMENT_REQUIRE__ &&
	(directRequire = directReplacement);

let renamedRequire = createRequire(import.meta.url);
const renamedReplacement = createRequire(import.meta.url);
globalThis.__RSPACK_USE_REPLACEMENT_REQUIRE__ &&
	(renamedRequire = (0, renamedReplacement));

export const conditionalCopyResolved = [
	directRequire.resolve("path"),
	renamedRequire.resolve("path")
];
