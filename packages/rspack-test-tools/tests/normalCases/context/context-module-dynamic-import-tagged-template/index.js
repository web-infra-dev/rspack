it("context module + dynamic import + tagged template", function (done) {
	const a = "child/index";
	import(String.raw`./${a}.js`).then(module => {
		expect(module.value).toBe("dynamic");
	});

	const tagFunc = function () {
		return "./child/index";
	};
	import(tagFunc`./${a}.js`).catch(err => {
		expect(err.message).toMatch(/Cannot find module/);
		done();
	});
});
