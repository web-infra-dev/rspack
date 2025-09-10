it("should ensure the combination for `raw` and `string` content", () => {
	expect(require("./lib?case-1")).toEqual([true, false, true]);
	expect(require("./lib?case-2")).toEqual([false, true, false]);
});
