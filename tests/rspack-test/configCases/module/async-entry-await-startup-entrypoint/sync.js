export function then() {
	throw new Error("a synchronous entry must not be awaited");
}
