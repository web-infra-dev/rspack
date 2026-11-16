it("accepts 1.2.3 for ~1", async () => {
	const lib = await import("tilde-lib");
	expect(lib.default.name).toBe("tilde-lib");
});

it("accepts 0.5.1 for ^0", async () => {
	const lib = await import("caret-zero-lib");
	expect(lib.default.name).toBe("caret-zero-lib");
});

it("accepts 0.0.4 for ^0.0", async () => {
	const lib = await import("caret-zero-zero-lib");
	expect(lib.default.name).toBe("caret-zero-zero-lib");
});
