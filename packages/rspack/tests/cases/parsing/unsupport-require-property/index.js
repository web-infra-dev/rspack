it("should transform unsupported require api to undefined", function () {
	expect(require.extensions).toBeUndefined();
	expect(require.ensure).toBeUndefined();
	expect(require.config).toBeUndefined();
	expect(require.vesrion).toBeUndefined();
	expect(require.amd).toBeUndefined();
	expect(require.include).toBeUndefined();
	expect(require.onError).toBeUndefined();

	expect(require.include("a")).toBeUndefined();
	expect(
		require.ensure(["a", "b"], function (require) {
			/* ... */
		})
	).toBeUndefined();
	expect(require.onError(function () {})).toBeUndefined();
});
