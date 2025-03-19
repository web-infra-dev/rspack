it("should work", function() {
	return import("./module").then(function(module) {
		expect(module.result).toEqual(42);
	});
});
