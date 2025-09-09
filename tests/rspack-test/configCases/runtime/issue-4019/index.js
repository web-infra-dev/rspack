it("load dynamic chunk", function (done) {
	import("./dynamic").then(module => {
		expect(module.value).toBe(1);
		done();
	});
});
