const __rspack_createRequire = "local-createRequire";
const __rspack_createRequire_require = "local-require";

export async function loadFs() {
	const fs = await import("fs");
	return [
		typeof fs.readFile,
		typeof fs.default.readFile,
		__rspack_createRequire,
		__rspack_createRequire_require
	];
}

it("should keep lazy node-commonjs helper names from colliding", async () => {
	expect(await loadFs()).toEqual([
		"function",
		"function",
		"local-createRequire",
		"local-require"
	]);
});
