it("should work with chunk loading false", function () {
	import("./dynamic").then(module => {
		expect(module.value).toBe(1);
	});
});
