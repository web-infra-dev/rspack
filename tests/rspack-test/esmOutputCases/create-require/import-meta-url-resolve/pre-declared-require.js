import { createRequire } from "node:module";

export function getRequire() {
	return req;
}

const req = createRequire(import.meta.url);

export function getNestedRequire() {
	function readRequire() {
		return nestedReq;
	}

	const nestedReq = createRequire(import.meta.url);
	return readRequire();
}
