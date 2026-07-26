it("should rewalk parallel sibling blocks across entries when available modules change", async () => {
	const first = await (await import("./module-b")).default;
	const second = await import(
		/* webpackChunkName: "parallel-module" */ "./module-a"
	);

	expect(first.default).toBe("module-c");
	expect(second.values).toEqual([
		"a:0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15:a",
		"b:0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15:a",
		"c:0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15:a",
		"d:0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15:a"
	]);
});
