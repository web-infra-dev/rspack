it("load dynamic chunk with hash", function (done) {
	import("./dynamic").then(module => {
		expect(module.value).toBe("dynamic");
		done();
	});
});
