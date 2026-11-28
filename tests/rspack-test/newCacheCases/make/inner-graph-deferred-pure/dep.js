export function pureFn() {
	console.log("inner-graph-impure");
	return "impure";
}
---
export function pureFn() {
	return "pure";
}
