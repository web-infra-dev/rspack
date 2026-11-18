it("should load chunks whose content hash is not numeric only", async () => {
	const modules = await Promise.all([
		import("./chunk?0"),
		import("./chunk?1"),
		import("./chunk?2"),
		import("./chunk?3"),
		import("./chunk?4"),
		import("./chunk?5"),
		import("./chunk?6"),
		import("./chunk?7")
	]);
	expect(modules.map(m => m.default)).toEqual([
		"?0",
		"?1",
		"?2",
		"?3",
		"?4",
		"?5",
		"?6",
		"?7"
	]);
});
