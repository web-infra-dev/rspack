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

export const mutationUnknowns = [
	typeof updateLater,
	typeof iterateLater,
	updatedRequire.a,
	iteratedRequire.a
];
