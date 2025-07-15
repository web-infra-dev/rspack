it("context module(sync) + cjs require + tagged template", function (done) {
	const a = "child/index";
	const module = require(String.raw`./${a}.js`);
	expect(module.value).toBe("dynamic");

	const tagFunc = function () {
		return "fail";
	};
	expect(function () {
		require(tagFunc`./${a}.js`);
	}).toThrowError(/Cannot find module/);
	done();
});
