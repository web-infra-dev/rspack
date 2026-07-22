import { createRequire } from "node:module";

export function load() {
	return req("./libCssExtractLoader.js");
}

const req = createRequire(import.meta.url);
