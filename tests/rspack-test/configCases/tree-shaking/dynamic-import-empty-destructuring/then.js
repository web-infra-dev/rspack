export const value = "then-receiver-kept";

export function then(resolve) {
	globalThis.__rspack_empty_destructuring_then__ = this.value;
	resolve({});
}
