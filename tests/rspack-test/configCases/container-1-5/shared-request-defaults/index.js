it("should consume via the resolved request", async () => {
	expect(await import("lib")).toEqual(
		expect.objectContaining({ default: "lib" })
	);
	expect(await import("other")).toEqual(
		expect.objectContaining({ default: "other-impl" })
	);
});

it("should provide the module the consumer falls back to", async () => {
	const provided =
		__webpack_require__.initializeSharingData.scopeToSharingDataMapping.default;
	const names = provided.map((item) => item.name).sort();
	// import omitted: provider follows `request`, not the config key;
	// explicit import: provider follows `import`, not `request`;
	// import: false: consumed only, never provided.
	expect(names).toEqual(["lib", "other"]);
	const lib = provided.find((item) => item.name === "lib");
	expect(lib.version).toBe("1.0.0");
	expect((await lib.factory())()).toEqual(
		expect.objectContaining({ default: "lib" })
	);
	const other = provided.find((item) => item.name === "other");
	expect(other.version).toBe("2.0.0");
	expect((await other.factory())()).toEqual(
		expect.objectContaining({ default: "other-impl" })
	);
});
