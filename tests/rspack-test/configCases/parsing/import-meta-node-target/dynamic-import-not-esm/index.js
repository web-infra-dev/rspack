const topLevelThis = this;

function getThis() {
	return this;
}

it("should not detect an ES module from dynamic import alone", async () => {
	expect(getThis()).not.toBe(undefined);
	expect(topLevelThis).not.toBe(undefined);
	expect((await import("./value.js")).default).toBe(42);
});
