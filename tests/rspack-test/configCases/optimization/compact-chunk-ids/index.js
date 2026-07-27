it("should load chunks with compact ids", async () => {
	const values = await Promise.all([
		import("./a"),
		import("./b"),
		import("./c")
	]);
	expect(values.map(value => value.default)).toEqual(["a", "b", "c"]);
});
