it("should handle circular chunks correctly", async function() {
	const result = await import(/* webpackChunkName: "a" */"./module-a");
	const result2 = await result.default();
	expect(result2.default()).toBe("x");
	const couldBe = function() {
		return import(/* webpackChunkName: "b" */"./module-b");
	};
});
