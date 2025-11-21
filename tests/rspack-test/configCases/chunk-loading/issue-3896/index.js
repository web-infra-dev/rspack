it("should work with chunk loading false", function () {
	const file = "a.js";
	import(`./file/${file}`).then(module => {
		expect(module.default).toBe("a");
	});
});
