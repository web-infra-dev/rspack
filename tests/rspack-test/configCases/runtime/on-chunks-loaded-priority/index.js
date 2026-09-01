if (globalThis.__neverLoaded) import("./lazy");

it("should run an even-priority handler without waiting for a blocked lower priority", () => {
	expect(globalThis.__onChunksLoadedOrder).toEqual(["even"]);
});

it("should import a CommonJS namespace", async () => {
	const namespace = await import("./commonjs");
	expect(namespace.default).toBe("commonjs");
});
