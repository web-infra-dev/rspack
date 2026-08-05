it("should not have import.meta.env", function() {
	var _env;
	(_env = import.meta.env) === null || _env === void 0 ? void 0 : _env.production;
	expect(_env).toBe(undefined);
});

it("should not warn for import.meta defined by DefinePlugin", function() {
	expect(import.meta.MY_ENV).toBe("canary");
});

if (FOO) {
	require("fail");
}
