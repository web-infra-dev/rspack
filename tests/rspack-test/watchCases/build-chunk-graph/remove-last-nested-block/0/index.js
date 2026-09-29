const loadOther = require("./keep");

it("should remove and restore the last nested async chunk", async () => {
	const value = await new Promise((resolve, reject) => {
		require.ensure(["./a"], () => {
			expect(require("./a")).toBe(1);
			require.ensure(["./b"], () => resolve(require("./b")), "inner");
		}, error => reject(error), "outer");
	});
	expect(value).toBe(42);
	expect(await loadOther()).toEqual([42, 43]);
});
