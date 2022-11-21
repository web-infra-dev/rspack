it("load dynamic chunk", function () {
	import("./dynamic").then(module => {
		expect(module.value).toBe("dynamic");
	});
});
