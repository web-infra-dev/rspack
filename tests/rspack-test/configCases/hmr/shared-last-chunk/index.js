it("does not combine different groups ending in the same chunk", async () => {
	const requested = [];
	__webpack_require__.f.record = id => requested.push(id);
	const b = await import("./b");
	expect(b.default).toBe("b");
	expect(requested).toEqual(["shared"]);
	requested.length = 0;
	const a = await import("./a");
	expect(a.default).toBe("a:x");
	expect(requested.sort()).toEqual(["shared", "x"]);
});
