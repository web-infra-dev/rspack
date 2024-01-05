it("should transform unsupported require api to undefined", function () {
	expect(require.extensions).toBeUndefined();
	expect(require.ensure).toBeUndefined();
	expect(require.config).toBeUndefined();
	expect(require.vesrion).toBeUndefined();
	expect(require.amd).toBeUndefined();
	expect(require.include).toBeUndefined();
	expect(require.onError).toBeUndefined();
	expect(require.main.require).toBeUndefined();
	expect(module.parent.require).toBeUndefined();

	expect(require.include("a")).toBeUndefined();
	expect(
		require.ensure(["a", "b"], function (require) {
			/* ... */
		})
	).toBeUndefined();
	expect(require.onError(function () {})).toBeUndefined();
	expect(require.main.require("a")).toBeUndefined();
	expect(module.parent.require("a")).toBeUndefined();

	function requireInBlock() {
		var require = {
			extensions: {},
			ensure: {},
			config: {},
			vesrion: {},
			amd: {},
			include: {},
			onError: {},
			main: {
				require: {}
			}
		};
		var module = {
			parent: {
				require: {}
			}
		};
		expect(require.extensions).toBeTruthy();
		expect(require.ensure).toBeTruthy();
		expect(require.config).toBeTruthy();
		expect(require.vesrion).toBeTruthy();
		expect(require.amd).toBeTruthy();
		expect(require.include).toBeTruthy();
		expect(require.onError).toBeTruthy();
		expect(require.main.require).toBeTruthy();
		expect(module.parent.require).toBeTruthy();
	}
	requireInBlock();
});
