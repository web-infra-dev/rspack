// Regression test for https://github.com/web-infra-dev/rspack/issues/16695
it("should not shadow variables with preserved class names", () => {
	const cache = new WeakMap();
	class e {
		static lookup(key) {
			return cache.get(key);
		}
	}
	cache.set(globalThis, "found");
	expect(e.lookup(globalThis)).toBe("found");
	expect(e.name).toBe("e");
});
