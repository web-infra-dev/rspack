import { createRequire } from "node:module";

function identity(value) {
	return value;
}

export const inlineEscapedRequireType = typeof identity(
	createRequire(import.meta.url)
);
