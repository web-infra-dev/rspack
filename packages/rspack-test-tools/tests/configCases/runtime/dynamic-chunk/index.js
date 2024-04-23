it("load dynamic chunk", function (done) {
	import("./dynamic").then(module => {
		expect(module.value).toBe("dynamic");
		done();
	});
});
