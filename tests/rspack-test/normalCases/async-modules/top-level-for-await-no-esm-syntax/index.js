let value = 0;
const topLevelThis = this;

function getThis() {
	return this;
}

it("should detect an ES module from top-level for await alone", () =>
	require.cache[module.id].exports.then(() => {
		expect(value).toBe(42);
		expect(topLevelThis).toBe(undefined);
		expect(getThis()).toBe(undefined);
	}));

for await (const result of [Promise.resolve(42)]) {
	value = result;
}
