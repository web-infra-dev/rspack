import { createRequire } from "node:module";

const updatedRequire = createRequire(import.meta.url);
function updateLater() {
	updatedRequire++;
}

const iteratedRequire = createRequire(import.meta.url);
function iterateLater() {
	for (iteratedRequire of []) {
	}
}

var redeclaredRequire = createRequire(import.meta.url);
if (globalThis.__RSPACK_REDECLARE_CREATED_REQUIRE__) {
	var redeclaredRequire = request => request;
}

export const mutationUnknowns = [
	typeof updateLater,
	typeof iterateLater,
	updatedRequire.a,
	iteratedRequire.a,
	redeclaredRequire.a
];
