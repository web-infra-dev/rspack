it("should not have import.meta.env", function() {
	var _env;
	(_env = import.meta.env) === null || _env === void 0 ? void 0 : _env.production;
	expect(_env).toBe(undefined);
});
