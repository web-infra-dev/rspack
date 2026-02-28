it("should decorate commonjs module with node module decorator when define property", function () {
	expect(__webpack_module__.children).toBeTruthy();
	expect(function () {
		Object.defineProperty(module, "exports", { value: 1 });
	}).not.toThrowError();
});
