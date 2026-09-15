it("should load wasm with an encoded hash in the filename", function() {
	return import("./module").then(function(module) {
		const result = module.run();
		expect(result).toEqual(84);
	});
});
